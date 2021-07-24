// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"context"
	"fmt"

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
	proofResponse, err := co.API().RPC.MMR.GenerateProof(blockNumber, blockHash)
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

type Head struct {
	LeafIndex  int // order in which this head was returned from the storage query
	ParaID uint32
	Data   []byte
}

func (co *Connection) FetchParaHeads(blockHash types.Hash) (map[uint32]Head, error) {

	keyPrefix := types.CreateStorageKeyPrefix("Paras", "Heads")

	keys, err := co.API().RPC.State.GetKeys(keyPrefix, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain keys")
		return nil, err
	}

	changeSets, err := co.API().RPC.State.QueryStorageAt(keys, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain headers")
		return nil, err
	}

	heads := make(map[uint32]Head)

	for _, changeSet := range changeSets {
		for index, change := range changeSet.Changes {
			if change.StorageData.IsNone() {
				continue
			}

			var paraID uint32

			if err := types.DecodeFromBytes(change.StorageKey[40:], &paraID); err != nil {
				co.log.WithError(err).Error("Failed to decode parachain ID")
				return nil, err
			}

			_, headDataWrapped := change.StorageData.Unwrap()

			var headData types.Bytes
			if err := types.DecodeFromBytes(headDataWrapped, &headData); err != nil {
				co.log.WithError(err).Error("Failed to decode HeadData wrapper")
				return nil, err
			}

			heads[paraID] = Head{
				LeafIndex: index,
				ParaID:    paraID,
				Data:      headData,
			}
		}
	}

	return heads, nil
}

// Fetch the latest block of a parachain that has been finalized at a relay chain block hash
func (co *Connection) FetchFinalizedParaHead(relayBlockhash types.Hash, paraID uint32) (*types.Header, error) {
	heads, err := co.FetchParaHeads(relayBlockhash)
	if err != nil {
		co.log.WithError(err).Error("Failed to fetch parachain heads from relay chain")
		return nil, err
	}

	if _, ok := heads[paraID]; !ok {
		return nil, fmt.Errorf("chain is not a registered parachain")
	}

	var header types.Header
	if err := types.DecodeFromBytes(heads[paraID].Data, &header); err != nil {
		co.log.WithError(err).Error("Failed to decode Header")
		return nil, err
	}

	return &header, nil
}
