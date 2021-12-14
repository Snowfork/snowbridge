// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"

	"github.com/sirupsen/logrus"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

type Connection struct {
	endpoint    string
	kp          *signature.KeyringPair
	api         *gsrpc.SubstrateAPI
	metadata    types.Metadata
	genesisHash types.Hash
}

func (co *Connection) API() *gsrpc.SubstrateAPI {
	return co.api
}

func (co *Connection) Metadata() *types.Metadata {
	return &co.metadata
}

func (co *Connection) Keypair() *signature.KeyringPair {
	return co.kp
}

func NewConnection(endpoint string, kp *signature.KeyringPair) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
	}
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

	log.WithFields(logrus.Fields{
		"endpoint":    co.endpoint,
		"metaVersion": meta.Version,
	}).Info("Connected to chain")

	return nil
}

func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}

func (co *Connection) GenesisHash() types.Hash {
	return co.genesisHash
}

func (co *Connection) GetFinalizedHeader() (*types.Header, error) {
	finalizedHash, err := co.api.RPC.Chain.GetFinalizedHead()
	if err != nil {
		return nil, err
	}

	finalizedHeader, err := co.api.RPC.Chain.GetHeader(finalizedHash)
	if err != nil {
		return nil, err
	}

	return finalizedHeader, nil
}

func (co *Connection) GetLatestBlockNumber() (*types.BlockNumber, error) {
	latestBlock, err := co.api.RPC.Chain.GetBlockLatest()
	if err != nil {
		return nil, err
	}

	return &latestBlock.Block.Header.Number, nil
}

func (co *Connection) GetDataForDigestItem(digestItem *AuxiliaryDigestItem) (types.StorageDataRaw, error) {
	storageKey, err := MakeStorageKey(digestItem.AsCommitment.ChannelID, digestItem.AsCommitment.Hash)
	if err != nil {
		return nil, err
	}

	data, err := co.API().RPC.Offchain.LocalStorageGet(offchain.Persistent, storageKey)
	if err != nil {
		log.WithError(err).Error("Failed to read commitment from offchain storage")
		return nil, err
	}

	if data != nil {
		log.WithFields(logrus.Fields{
			"commitmentSizeBytes": len(*data),
		}).Debug("Retrieved commitment from offchain storage")
	} else {
		log.WithError(err).Error("Commitment not found in offchain storage")
		return nil, err
	}

	return *data, nil
}

func (co *Connection) ReadBasicOutboundMessages(digestItem AuxiliaryDigestItem) (
	BasicOutboundChannelMessages, error) {
	data, err := co.GetDataForDigestItem(&digestItem)
	if err != nil {
		return nil, err
	}

	var messages []BasicOutboundChannelMessage

	err = types.DecodeFromBytes(data, &messages)
	if err != nil {
		log.WithError(err).Error("Failed to decode commitment messages")
		return nil, err
	}

	return messages, nil
}

func (co *Connection) ReadIncentivizedOutboundMessages(digestItem AuxiliaryDigestItem) (
	IncentivizedOutboundChannelMessages, error) {
	data, err := co.GetDataForDigestItem(&digestItem)
	if err != nil {
		return nil, err
	}

	var messages []IncentivizedOutboundChannelMessage

	err = types.DecodeFromBytes(data, &messages)
	if err != nil {
		log.WithError(err).Error("Failed to decode commitment messages")
		return nil, err
	}

	return messages, nil
}
