// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"context"
	"fmt"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

type Connection struct {
	endpoint    string
	api         *gsrpc.SubstrateAPI
	metadata    types.Metadata
	genesisHash types.Hash
}

func NewConnection(endpoint string) *Connection {
	return &Connection{
		endpoint: endpoint,
	}
}

func (co *Connection) API() *gsrpc.SubstrateAPI {
	return co.api
}

func (co *Connection) Metadata() *types.Metadata {
	return &co.metadata
}

func (co *Connection) Connect(_ context.Context) error {
	// Initialize API
	api, err := gsrpc.NewSubstrateAPI(co.endpoint)
	if err != nil {
		return err
	}
	co.api = api

	// Fetch metadata
	meta, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}
	co.metadata = *meta

	// Fetch genesis hash
	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}
	co.genesisHash = genesisHash

	log.WithFields(log.Fields{
		"endpoint":    co.endpoint,
		"metaVersion": meta.Version,
	}).Info("Connected to chain")

	return nil
}

func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}

func (co *Connection) GenerateProofForBlock(
	blockNumber uint64,
	latestBeefyBlockHash types.Hash,
	beefyActivationBlock uint64,
) (types.GenerateMMRProofResponse, error) {
	log.WithFields(log.Fields{
		"blockNumber": blockNumber,
		"blockHash":   latestBeefyBlockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")

	// We expect 1 mmr leaf for each block. MMR leaf indexes start from 0, but block numbers start from 1,
	// so the mmr leaf index should be 1 less than the block number.
	// However, some chains only started using beefy late in their existence, so there are no leafs for
	// blocks produced before beefy was activated. We subtract the block in which beefy was activated on the
	// chain to account for this.
	//
	// LeafIndex(currentBlock, activationBlock) := currentBlock - Max(activationBlock, 1)
	//
	// Example: LeafIndex(5, 3) = 2
	//
	// Block Number: 1 -> 2 -> 3 -> 4 -> 5
	// Leaf Index:             0 -> 1 -> 2
	//                         ^         ^
	//                         |         |
	//                         |         Leaf we want
	//                         |
	//                         Activation Block
	//
	var leafIndex uint64
	if beefyActivationBlock == 0 {
		leafIndex = blockNumber - 1
	} else {
		leafIndex = blockNumber - beefyActivationBlock
	}

	proofResponse, err := co.API().RPC.MMR.GenerateProof(leafIndex, latestBeefyBlockHash)
	if err != nil {
		return types.GenerateMMRProofResponse{}, err
	}

	var proofItemsHex = []string{}
	for _, item := range proofResponse.Proof.Items {
		proofItemsHex = append(proofItemsHex, item.Hex())
	}

	log.WithFields(log.Fields{
		"BlockHash":                       proofResponse.BlockHash.Hex(),
		"Leaf.ParentNumber":               proofResponse.Leaf.ParentNumberAndHash.ParentNumber,
		"Leaf.Hash":                       proofResponse.Leaf.ParentNumberAndHash.Hash.Hex(),
		"Leaf.ParachainHeads":             proofResponse.Leaf.ParachainHeads.Hex(),
		"Leaf.BeefyNextAuthoritySet.ID":   proofResponse.Leaf.BeefyNextAuthoritySet.ID,
		"Leaf.BeefyNextAuthoritySet.Len":  proofResponse.Leaf.BeefyNextAuthoritySet.Len,
		"Leaf.BeefyNextAuthoritySet.Root": proofResponse.Leaf.BeefyNextAuthoritySet.Root.Hex(),
		"Proof.LeafIndex":                 proofResponse.Proof.LeafIndex,
		"Proof.LeafCount":                 proofResponse.Proof.LeafCount,
		"Proof.Items":                     proofItemsHex,
	}).Info("Generated MMR Proof")

	return proofResponse, nil
}

type ParaHead struct {
	ParaID uint32
	Data   types.Bytes
}

// Offset of encoded para id in storage key.
// The key is of this format:
//   ParaId: u32
//   Key: hash_twox_128("Paras") + hash_twox_128("Heads") + hash_twox_64(ParaId) + Encode(ParaId)
const ParaIDOffset = 16 + 16 + 8

func (co *Connection) FetchParaHeads(blockHash types.Hash) (map[uint32]ParaHead, error) {

	keyPrefix := types.CreateStorageKeyPrefix("Paras", "Heads")

	keys, err := co.fetchKeys(keyPrefix, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to get all parachain keys")
		return nil, err
	}

	log.WithFields(log.Fields{
		"numKeys":          len(keys),
		"storageKeyPrefix": fmt.Sprintf("%#x", keyPrefix),
		"block":            blockHash.Hex(),
	}).Trace("Found keys for Paras.Heads storage map")

	changeSets, err := co.API().RPC.State.QueryStorageAt(keys, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to get all parachain headers")
		return nil, err
	}

	heads := make(map[uint32]ParaHead)

	for _, changeSet := range changeSets {
		for _, change := range changeSet.Changes {
			if change.StorageData.IsNone() {
				continue
			}

			var paraID uint32
			if err := types.DecodeFromBytes(change.StorageKey[ParaIDOffset:], &paraID); err != nil {
				log.WithError(err).Error("Failed to decode parachain ID")
				return nil, err
			}

			_, headDataWrapped := change.StorageData.Unwrap()

			var headData types.Bytes
			if err := types.DecodeFromBytes(headDataWrapped, &headData); err != nil {
				log.WithError(err).Error("Failed to decode HeadData wrapper")
				return nil, err
			}

			heads[paraID] = ParaHead{
				ParaID: paraID,
				Data:   headData,
			}
		}
	}

	return heads, nil
}

func (co *Connection) FetchFinalizedParaHead(relayBlockhash types.Hash, paraID uint32) (*types.Header, error) {
	encodedParaID, err := types.EncodeToBytes(paraID)
	if err != nil {
		return nil, err
	}

	storageKey, err := types.CreateStorageKey(co.Metadata(), "Paras", "Heads", encodedParaID, nil)
	if err != nil {
		return nil, err
	}

	var headerBytes types.Bytes
	ok, err := co.API().RPC.State.GetStorage(storageKey, &headerBytes, relayBlockhash)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, fmt.Errorf("parachain head not found")
	}

	var header types.Header
	if err := types.DecodeFromBytes(headerBytes, &header); err != nil {
		log.WithError(err).Error("Failed to decode Header")
		return nil, err
	}

	return &header, nil
}

func (co *Connection) FetchMMRLeafCount(relayBlockhash types.Hash) (uint64, error) {
	mmrLeafCountKey, err := types.CreateStorageKey(co.Metadata(), "Mmr", "NumberOfLeaves", nil, nil)
	if err != nil {
		return 0, err
	}
	var mmrLeafCount uint64

	ok, err := co.API().RPC.State.GetStorage(mmrLeafCountKey, &mmrLeafCount, relayBlockhash)
	if err != nil {
		return 0, err
	}

	if !ok {
		return 0, fmt.Errorf("MMR Leaf Count Not Found")
	}

	log.WithFields(log.Fields{
		"mmrLeafCount": mmrLeafCount,
	}).Info("MMR Leaf Count")

	return mmrLeafCount, nil
}

func (co *Connection) fetchKeys(keyPrefix []byte, blockHash types.Hash) ([]types.StorageKey, error) {
	const pageSize = 50
	var startKey *types.StorageKey

	if pageSize < 1 {
		return nil, fmt.Errorf("page size cannot be zero")
	}

	var results []types.StorageKey
	log.WithFields(log.Fields{
		"keyPrefix": keyPrefix,
		"blockHash": blockHash.Hex(),
		"pageSize":  pageSize,
	}).Trace("Fetching paged keys.")

	pageIndex := 0
	for {
		response, err := co.API().RPC.State.GetKeysPaged(keyPrefix, pageSize, startKey, blockHash)
		if err != nil {
			return nil, err
		}

		log.WithFields(log.Fields{
			"keysInPage": len(response),
			"pageIndex":  pageIndex,
		}).Trace("Fetched a page of keys.")

		results = append(results, response...)
		if uint32(len(response)) < pageSize {
			break
		} else {
			startKey = &response[len(response)-1]
			pageIndex++
		}
	}

	log.WithFields(log.Fields{
		"totalNumKeys":  len(results),
		"totalNumPages": pageIndex + 1,
	}).Trace("Fetching of paged keys complete.")

	return results, nil
}
