// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"context"
	"fmt"
	"sort"

	"github.com/sirupsen/logrus"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v3"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
)

type Connection struct {
	endpoint    string
	api         *gsrpc.SubstrateAPI
	metadata    types.Metadata
	genesisHash types.Hash
	log         *logrus.Entry
}

func NewConnection(endpoint string, log *logrus.Entry) *Connection {
	return &Connection{
		endpoint: endpoint,
		log:      log,
	}
}

func (co *Connection) GetAPI() *gsrpc.SubstrateAPI {
	return co.api
}

func (co *Connection) GetMetadata() *types.Metadata {
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

	co.log.WithFields(logrus.Fields{
		"endpoint":    co.endpoint,
		"metaVersion": meta.Version,
	}).Info("Connected to chain")

	return nil
}

func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}

func (co *Connection) GetMMRLeafForBlock(
	blockNumber uint64,
	blockHash types.Hash,
) (types.GenerateMMRProofResponse, error) {
	co.log.WithFields(logrus.Fields{
		"blockNumber": blockNumber,
		"blockHash":   blockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")
	proofResponse, err := co.GetAPI().RPC.MMR.GenerateProof(blockNumber, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to generate mmr proof")
		return types.GenerateMMRProofResponse{}, err
	}

	var proofItemsHex = []string{}
	for _, item := range proofResponse.Proof.Items {
		proofItemsHex = append(proofItemsHex, item.Hex())
	}

	co.log.WithFields(logrus.Fields{
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
	LeafIndex  int // order in which this head was returned from the storage query
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

	keys, err := co.GetAPI().RPC.State.GetKeys(keyPrefix, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain keys")
		return nil, err
	}

	changeSets, err := co.GetAPI().RPC.State.QueryStorageAt(keys, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain headers")
		return nil, err
	}

	heads := make(map[uint32]ParaHead)

	for _, changeSet := range changeSets {
		for index, change := range changeSet.Changes {
			if change.StorageData.IsNone() {
				continue
			}

			var paraID uint32
			if err := types.DecodeFromBytes(change.StorageKey[ParaIDOffset:], &paraID); err != nil {
				co.log.WithError(err).Error("Failed to decode parachain ID")
				return nil, err
			}

			_, headDataWrapped := change.StorageData.Unwrap()

			var headData types.Bytes
			if err := types.DecodeFromBytes(headDataWrapped, &headData); err != nil {
				co.log.WithError(err).Error("Failed to decode HeadData wrapper")
				return nil, err
			}

			heads[paraID] = ParaHead{
				LeafIndex: index,
				ParaID:    paraID,
				Data:      headData,
			}
		}
	}

	return heads, nil
}

// ByLeafIndex implements sort.Interface based on the LeafIndex field.
type ByLeafIndex []ParaHead
func (b ByLeafIndex) Len() int           { return len(b) }
func (b ByLeafIndex) Less(i, j int) bool { return b[i].LeafIndex < b[j].LeafIndex }
func (b ByLeafIndex) Swap(i, j int)      { b[i], b[j] = b[j], b[i] }

// AsProofInput transforms heads into a slice of head datas,
// in the original order they were returned by the Paras.Heads storage query.
func (co *Connection) AsProofInput(heads map[uint32]ParaHead) []types.Bytes {
	// make a slice of values in the map
	headsAsSlice := make([]ParaHead, len(heads))
	for i, v := range heads {
		headsAsSlice[i] = v
	}

	// sort by leaf index
	sort.Sort(ByLeafIndex(headsAsSlice))

	// map over slice to retrieve header data
    data := make([]types.Bytes, len(headsAsSlice))
    for i, h := range headsAsSlice {
        data[i] = h.Data
    }
    return data
}

func (co *Connection) FetchFinalizedParaHead(relayBlockhash types.Hash, paraID uint32) (*types.Header, error) {
	encodedParaID, err := types.EncodeToBytes(paraID)
	if err != nil {
		return nil, err
	}

	storageKey, err := types.CreateStorageKey(co.GetMetadata(), "Paras", "Heads", encodedParaID, nil)
	if err != nil {
		return nil, err
	}

	var header types.Header
	ok, err := co.GetAPI().RPC.State.GetStorage(storageKey, &header, relayBlockhash)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, fmt.Errorf("parachain head not found")
	}

	return &header, nil
}
