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

func (co *Connection) GetAllParaheadsWithOwn(blockHash types.Hash, ownParachainId uint32) (
	[]types.Bytes, int, types.Header, error) {
	none := types.NewOptionU32Empty()
	encoded, err := types.EncodeToBytes(none)
	if err != nil {
		co.log.WithError(err).Error("Error")
		return nil, 0, types.Header{}, err
	}

	baseParaHeadsStorageKey, err := types.CreateStorageKey(
		co.GetMetadata(),
		"Paras",
		"Heads", encoded, nil)
	if err != nil {
		co.log.WithError(err).Error("Failed to create parachain header storage key")
		return nil, 0, types.Header{}, err
	}

	keysResponse, err := co.GetAPI().RPC.State.GetKeys(baseParaHeadsStorageKey, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain keys")
		return nil, 0, types.Header{}, err
	}

	headersResponse, err := co.GetAPI().RPC.State.QueryStorage(keysResponse, blockHash, blockHash)
	if err != nil {
		co.log.WithError(err).Error("Failed to get all parachain headers")
		return nil, 0, types.Header{}, err
	}

	co.log.Info("Got all parachain headers")
	var headers []types.Bytes
	var ownParachainHeaderPos int
	for _, headerResponse := range headersResponse {
		for index, change := range headerResponse.Changes {

			// TODO fix this manual slice with a proper type decode. only the last few bytes are for the ParaId,
			// not sure what the early ones are for.
			key := change.StorageKey[40:]
			var parachainID types.U32
			if err := types.DecodeFromBytes(key, &parachainID); err != nil {
				co.log.WithError(err).Error("Failed to decode parachain ID")
				return nil, 0, types.Header{}, err
			}

			var headerBytes types.Bytes
			if err := types.DecodeFromBytes(change.StorageData, &headerBytes); err != nil {
				co.log.WithError(err).Error("Failed to decode MMREncodableOpaqueLeaf")
				return nil, 0, types.Header{}, err
			}
			headers = append(headers, headerBytes)

			if parachainID == types.U32(ownParachainId) {
				ownParachainHeaderPos = index
			}

		}
	}
	var ownParachainHeader types.Header
	if err := types.DecodeFromBytes(headers[ownParachainHeaderPos], &ownParachainHeader); err != nil {
		co.log.WithError(err).Error("Failed to decode Header")
		return nil, 0, types.Header{}, err
	}

	co.log.WithField("parachainId", ownParachainId).Info("Decoding header for own parachain")
	co.log.WithFields(logrus.Fields{
		"headerBytes":           fmt.Sprintf("%#x", headers[ownParachainHeaderPos]),
		"header.ParentHash":     ownParachainHeader.ParentHash.Hex(),
		"header.Number":         ownParachainHeader.Number,
		"header.StateRoot":      ownParachainHeader.StateRoot.Hex(),
		"header.ExtrinsicsRoot": ownParachainHeader.ExtrinsicsRoot.Hex(),
		"header.Digest":         ownParachainHeader.Digest,
		"parachainId":           ownParachainId,
	}).Info("Decoded header for parachain")

	return headers, ownParachainHeaderPos, ownParachainHeader, nil
}

// Fetch the latest block of a parachain that has been finalized at a relay chain block hash
func (co *Connection) FetchLatestFinalizedParaBlockNumber(relayBlockhash types.Hash, parachainId uint32) (uint64, error) {
	_, _, ownParaHead, err := co.GetAllParaheadsWithOwn(relayBlockhash, parachainId)

	if err != nil {
		co.log.WithError(err).Error("Failed to get parachain heads from relay chain")
		return 0, err
	}

	finalizedParaBlockNumber := uint64(ownParaHead.Number)

	return finalizedParaBlockNumber, nil
}
