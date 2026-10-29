//go:build forkfixture

package beefy

import (
	"context"
	"encoding/hex"
	"math/big"
	"os"
	"strconv"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

// Writes submitFinal arguments carrying a validator proof for every signer of a stored
// BEEFY justification. Fork tests pick the subset a new BeefyClient samples.
func TestWriteForkFixture(t *testing.T) {
	url := os.Getenv("POLKADOT_RPC")
	block, err := strconv.ParseUint(os.Getenv("BEEFY_BLOCK"), 10, 64)
	if url == "" || err != nil {
		t.Skip("set POLKADOT_RPC and BEEFY_BLOCK")
	}
	out := os.Getenv("FIXTURE_OUT")

	conn := relaychain.NewConnection(url, nil)
	if err := conn.Connect(context.Background()); err != nil {
		t.Fatal(err)
	}
	li := &PolkadotListener{conn: conn}

	hash, err := conn.API().RPC.Chain.GetBlockHash(block)
	if err != nil {
		t.Fatal(err)
	}
	signedBlock, err := conn.API().RPC.Chain.GetBlock(hash)
	if err != nil {
		t.Fatal(err)
	}
	var commitment *types.SignedCommitment
	for _, j := range signedBlock.Justifications {
		if j.EngineID() != "BEEF" {
			continue
		}
		sc := types.OptionalSignedCommitment{}
		if err := types.DecodeFromBytes(j.Payload(), &sc); err != nil {
			t.Fatal(err)
		}
		if ok, v := sc.Unwrap(); ok {
			commitment = &v
		}
	}
	if commitment == nil || uint64(commitment.Commitment.BlockNumber) != block {
		t.Fatalf("no BEEFY justification for block %d", block)
	}

	validators, err := li.queryBeefyAuthorities(hash)
	if err != nil {
		t.Fatal(err)
	}
	var leaves [][]byte
	for _, v := range validators {
		addr, err := v.IntoEthereumAddress()
		if err != nil {
			t.Fatal(err)
		}
		leaves = append(leaves, addr.Bytes())
	}
	_, root, _, err := merkle.GenerateMerkleProof(leaves, 0)
	if err != nil {
		t.Fatal(err)
	}

	req := Request{Validators: validators, SignedCommitment: *commitment}
	copy(req.ValidatorsRoot[:], root)

	var indices []uint64
	for i, sig := range commitment.Signatures {
		if sig.IsSome() {
			indices = append(indices, uint64(i))
		}
	}
	params, err := req.MakeSubmitFinalParams(indices, []*big.Int{})
	if err != nil {
		t.Fatal(err)
	}

	abi, err := contracts.BeefyClientMetaData.GetAbi()
	if err != nil {
		t.Fatal(err)
	}
	packed, err := abi.Methods["submitFinal"].Inputs.Pack(
		params.Commitment, params.Bitfield, params.Proofs, params.Leaf, [][32]byte{}, big.NewInt(0),
	)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(out, []byte("0x"+hex.EncodeToString(packed)), 0o644); err != nil {
		t.Fatal(err)
	}
	t.Logf("block %d set %d validators %d signers %d root %x -> %s",
		block, commitment.Commitment.ValidatorSetID, len(validators), len(indices), root, out)
}
