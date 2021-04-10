// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"context"
	"encoding/json"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/store"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	config        *Config
	conn          *Connection
	beefyMessages chan<- store.DatabaseCmd
	log           *logrus.Entry
}

func NewListener(config *Config, conn *Connection, beefyMessages chan<- store.DatabaseCmd, log *logrus.Entry) *Listener {
	return &Listener{
		config:        config,
		conn:          conn,
		beefyMessages: beefyMessages,
		log:           log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	if li.beefyMessages != nil {
		close(li.beefyMessages)
	}
	return ctx.Err()
}

func (li *Listener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.conn.api.Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case msg := <-ch:

			signedCommitment := &store.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Failed to decode BEEFY commitment messages")
			}

			li.log.Info("Witnessed a new BEEFY commitment: \n", msg.(string))
			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			signedCommitmentBytes, err := json.Marshal(signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Failed to marshal signed commitment:", signedCommitment)
				continue
			}

			// TODO:
			// beefyAuthorities, err := li.getBeefyAuthorities(uint64(signedCommitment.Commitment.BlockNumber))
			// if err != nil {
			// 	li.log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
			// }

			beefyAuthorities := []common.Address{
				common.HexToAddress("0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b"),
				common.HexToAddress("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"),
			}

			beefyAuthoritiesBytes, err := json.Marshal(beefyAuthorities)
			if err != nil {
				li.log.WithError(err).Error("Failed to marshal BEEFY authorities:", beefyAuthorities)
				continue
			}

			info := store.BeefyRelayInfo{
				ValidatorAddresses: beefyAuthoritiesBytes,
				SignedCommitment:   signedCommitmentBytes,
				Status:             store.CommitmentWitnessed,
			}

			li.log.Info("1: Writing BEEFY info to database with status 'WitnessedCommitment'")
			cmd := store.NewDatabaseCmd(&info, store.Create, nil)
			li.beefyMessages <- cmd
		}
	}
}

func (li *Listener) getBeefyAuthorities(blockNumber uint64) (substrate.Authorities, error) {
	blockHash, err := li.conn.api.RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return substrate.Authorities{}, err
	}

	meta, err := li.conn.api.RPC.State.GetMetadataLatest()
	if err != nil {
		return substrate.Authorities{}, err
	}

	storageKey, err := types.CreateStorageKey(meta, "Beefy", "Authorities", nil, nil)
	if err != nil {
		return substrate.Authorities{}, err
	}

	storageChangeSet, err := li.conn.api.RPC.State.QueryStorage([]types.StorageKey{storageKey}, blockHash, blockHash)
	if err != nil {
		return substrate.Authorities{}, err
	}

	authorities := substrate.Authorities{}
	for _, storageChange := range storageChangeSet {
		for _, keyValueOption := range storageChange.Changes {
			bz, err := keyValueOption.MarshalJSON()
			if err != nil {
				return substrate.Authorities{}, err
			}

			err = types.DecodeFromBytes(bz, &authorities)
			if err != nil {
				return substrate.Authorities{}, err
			}

		}
	}
	// TODO: Decode authorities using @polkadot/util-crypto/ethereum/encode.js ethereumEncode() method

	// if data != nil {
	// 	li.log.WithFields(logrus.Fields{
	// 		"block":               signedCommitment.Commitment.BlockNumber,
	// 		"commitmentSizeBytes": len(*data),
	// 	}).Debug("Retrieved authorities from storage")
	// } else {
	// 	li.log.WithError(err).Error("Authorities not found in storage")
	// 	continue
	// }

	return authorities, nil
}
