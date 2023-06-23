package beefy

import (
	"bytes"
	"fmt"
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

type InitialRequestParams struct {
	Commitment contracts.BeefyClientCommitment
	Bitfield   []*big.Int
	Proof      contracts.BeefyClientValidatorProof
}

type FinalRequestParams struct {
	Commitment     contracts.BeefyClientCommitment
	Bitfield       []*big.Int
	Proofs         []contracts.BeefyClientValidatorProof
	Leaf           contracts.BeefyClientMMRLeaf
	LeafProof      [][32]byte
	LeafProofOrder *big.Int
}

// Builds a payload which is partially SCALE-encoded. This is more efficient for the light client to verify
// as it does not have to implement a fully fledged SCALE-encoder.
func buildPayload(items []types.PayloadItem) (*contracts.BeefyClientPayload, error) {
	index := -1

	for i, payloadItem := range items {
		// MMR Root ID as "mh"
		// https://github.com/paritytech/substrate/blob/cbd8f1b56fd8ab9af0d9317432cc735264c89d70/primitives/beefy/src/payload.rs#L33
		if payloadItem.ID == [2]byte{0x6d, 0x68} {
			index = i
		}
	}

	// Contains one entry so index should be 0
	// https://github.com/paritytech/substrate/blob/cbd8f1b56fd8ab9af0d9317432cc735264c89d70/primitives/beefy/src/payload.rs#L48
	if index < 0 {
		return nil, fmt.Errorf("did not find mmr root hash in commitment")
	}

	mmrRootHash := [32]byte{}

	if len(items[index].Data) != 32 {
		return nil, fmt.Errorf("mmr root hash is invalid")
	}

	if copy(mmrRootHash[:], items[index].Data) != 32 {
		return nil, fmt.Errorf("mmr root hash is invalid")
	}

	payloadBytes, err := types.EncodeToBytes(items)
	if err != nil {
		return nil, err
	}

	// Trick here is that in payload of beefy commitment only MmrRootHash is required
	// so just split to some unknown prefix and suffix in order to reconstruct later
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/core/packages/contracts/contracts/BeefyClient.sol#L483-L492
	slices := bytes.Split(payloadBytes, mmrRootHash[:])
	if len(slices) != 2 {
		// Its theoretically possible that the payload items may contain mmrRootHash more than once, causing an invalid split
		return nil, fmt.Errorf("expected 2 slices")
	}

	return &contracts.BeefyClientPayload{
		MmrRootHash: mmrRootHash,
		Prefix:      slices[0],
		Suffix:      slices[1],
	}, nil
}
