package beefy

import (
	"testing"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"github.com/stretchr/testify/assert"
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

func buildSignature(v uint8, r [32]byte, s [32]byte) (signature types.BeefySignature) {
	var input []byte
	input = append(r[:], s[:]...)
	input = append(input, v)
	copy(signature[:], input)
	return signature
}
