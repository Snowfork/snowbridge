package fisherman

// package mmr

// // TODO: move to go-substrate-rpc-client

import (
// "github.com/snowfork/go-substrate-rpc-client/v4/client"
// "github.com/snowfork/go-substrate-rpc-client/v4/types"
)

func zero() uint32 {
	return 0
}

// // GenerateMMRProofResponse contains the generate proof rpc response
// type GenerateMMRProofResponse struct {
// 	BlockHash H256
// 	Leaf      MMRLeaf
// 	Proof     MMRProof
// }

// // GenerateAncestryProof retrieves a MMR ancestry proof for the specified block number, at the given blockHash (useful to query a proof at an earlier block, likely with another MMR root)
// func (c *MMR) GenerateAncestryProof(blockNumber uint32, blockHash types.Hash) (types.GenerateMMRProofResponse, error) {
// 	return c.generateProof(blockNumber, &blockHash)
// }

// // GenerateProofLatest retrieves the latest MMR proof and leaf for the specified leave index
// func (c *MMR) GenerateProofLatest(blockNumber uint32) (types.GenerateMMRProofResponse, error) {
// 	return c.generateAncestryProof(blockNumber, nil)
// }

// func (c *MMR) generateProof(blockNumber uint32, blockHash *types.Hash) (types.GenerateMMRProofResponse, error) {
// 	var res types.GenerateMMRProofResponse
// 	block := uint32{blockNumber}
// 	err := client.CallWithBlockHash(c.client, &res, "mmr_generateAncestryProof", blockHash, block, nil)
// 	if err != nil {
// 		return types.GenerateMMRProofResponse{}, err
// 	}

// 	return res, nil
// }
