package beefy

import (
	"testing"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"github.com/snowfork/snowbridge/relayer/substrate"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCleanSignatureNochange(t *testing.T) {
	hash, _ := util.HexStringTo32Bytes("0x3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0")
	r, _ := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, _ := util.HexStringTo32Bytes("0x6dc0d1a7743c3328bfcfe05a2f8691e114f9143776a461ddad6e8b858bb19c1d")
	v := uint8(1)
	signature := buildSignature(v, r, s)
	publicKey, err := crypto.Ecrecover(hash[:], signature[:])
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, len(publicKey), 65)
	vAfter, rAfter, sAfter, reverted, err := CleanSignature(signature)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, reverted, false)
	assert.Equal(t, vAfter, v+27)
	assert.Equal(t, rAfter, r)
	assert.Equal(t, sAfter, s)
}

func TestCleanSignatureWithSConverted(t *testing.T) {
	hash, _ := util.HexStringTo32Bytes("0x3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0")
	r, _ := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, _ := util.HexStringTo32Bytes("0x6dc0d1a7743c3328bfcfe05a2f8691e114f9143776a461ddad6e8b858bb19c1d")
	v := uint8(1)
	signature := buildSignature(v, r, s)
	publicKey, err := crypto.Ecrecover(hash[:], signature[:])
	if err != nil {
		t.Fatal(err)
	}
	negativeS, _ := util.HexStringTo32Bytes("0x923f2e588bc3ccd740301fa5d0796e1da5b5c8af38a43e5e1263d3074484a524")
	negativeV := byte(0)
	negativeSignature := buildSignature(negativeV, r, negativeS)
	vAfter, rAfter, sAfter, reverted, err := CleanSignature(negativeSignature)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, reverted, true)
	assert.Equal(t, vAfter, v+27)
	assert.Equal(t, rAfter, r)
	assert.Equal(t, sAfter, s)
	publicKeyAfter, err := crypto.Ecrecover(hash[:], negativeSignature[:])
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, len(publicKeyAfter), 65)
	assert.Equal(t, publicKey, publicKeyAfter)
}

// validAuthority returns a BEEFY authority (compressed ECDSA pubkey) together
// with its Ethereum address.
func validAuthority(t *testing.T) (substrate.Authority, []byte) {
	t.Helper()
	priv, err := crypto.GenerateKey()
	require.NoError(t, err)
	compressed := crypto.CompressPubkey(&priv.PublicKey)
	require.Len(t, compressed, 33)
	var authority substrate.Authority
	copy(authority[:], compressed)
	address := crypto.PubkeyToAddress(priv.PublicKey)
	return authority, address.Bytes()
}

// invalidAuthority returns an authority whose key cannot be decompressed into an
// Ethereum address, exercising the failed-conversion sentinel path.
func invalidAuthority(t *testing.T) substrate.Authority {
	t.Helper()
	var authority substrate.Authority
	for i := range authority {
		authority[i] = 0xff
	}
	_, err := authority.IntoEthereumAddress()
	require.Error(t, err, "0xff authority should fail to convert")
	return authority
}

// TestGenerateValidatorAddressProofFailedConversion asserts that a BEEFY
// authority that cannot be converted to an Ethereum address contributes a
// 20-byte zero leaf, matching the runtime's FAILED_BEEFY_TO_ETH_ADDRESS sentinel
// (paritytech/polkadot-sdk#12214). Building the validator-set root the same way
// must let us reconstruct a matching root and generate a proof.
func TestGenerateValidatorAddressProofFailedConversion(t *testing.T) {
	val0, addr0 := validAuthority(t)
	val1, addr1 := validAuthority(t)
	failed := invalidAuthority(t)
	val3, addr3 := validAuthority(t)

	validators := []substrate.Authority{val0, val1, failed, val3}

	// Reconstruct the leaves exactly as pallet-beefy-mmr does: real addresses
	// for convertible keys, a 20-byte zero sentinel for the failed one.
	leaves := [][]byte{addr0, addr1, make([]byte, 20), addr3}
	_, root, _, err := merkle.GenerateMerkleProof(leaves, 0)
	require.NoError(t, err)

	var validatorsRoot [32]byte
	copy(validatorsRoot[:], root)

	request := &Request{
		Validators:     validators,
		ValidatorsRoot: validatorsRoot,
	}

	// A proof can be generated for a signing (convertible) validator because the
	// reconstructed root matches the on-chain root.
	for _, idx := range []int64{0, 1, 3} {
		_, err := request.generateValidatorAddressProof(idx)
		assert.NoErrorf(t, err, "proof for validator %d should match root", idx)
	}
}

// TestGenerateValidatorAddressProofEmptyLeafMismatch documents why the parity
// change matters: the old behavior used an empty leaf for failed conversions,
// which no longer matches a root committed by a runtime that includes
// paritytech/polkadot-sdk#12214 (a 20-byte zero leaf).
func TestGenerateValidatorAddressProofEmptyLeafMismatch(t *testing.T) {
	val0, addr0 := validAuthority(t)
	failed := invalidAuthority(t)

	validators := []substrate.Authority{val0, failed}

	// Root committed by the post-#12214 runtime: 20-byte zero sentinel leaf.
	postUpgradeLeaves := [][]byte{addr0, make([]byte, 20)}
	_, postUpgradeRoot, _, err := merkle.GenerateMerkleProof(postUpgradeLeaves, 0)
	require.NoError(t, err)

	// Root the old relayer would have produced: empty sentinel leaf.
	oldLeaves := [][]byte{addr0, {}}
	_, oldRoot, _, err := merkle.GenerateMerkleProof(oldLeaves, 0)
	require.NoError(t, err)

	assert.NotEqual(t, postUpgradeRoot, oldRoot,
		"empty and 20-byte zero sentinels must produce different roots")

	var validatorsRoot [32]byte
	copy(validatorsRoot[:], postUpgradeRoot)
	request := &Request{Validators: validators, ValidatorsRoot: validatorsRoot}

	// With the parity fix the reconstructed root matches the post-upgrade root.
	_, err = request.generateValidatorAddressProof(0)
	assert.NoError(t, err)
}

func buildSignature(v uint8, r [32]byte, s [32]byte) (signature types.BeefySignature) {
	var input []byte
	input = append(r[:], s[:]...)
	input = append(input, v)
	copy(signature[:], input)
	return signature
}
