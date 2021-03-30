// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"context"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	rpcOffchain "github.com/snowfork/go-substrate-rpc-client/v2/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	config        *Config
	conn          *Connection
	relayconn     *Connection
	messages      chan<- []chain.Message
	beefyMessages chan<- store.DatabaseCmd
	log           *logrus.Entry
}

func NewListener(config *Config, conn *Connection, relayconn *Connection, messages chan<- []chain.Message,
	beefyMessages chan<- store.DatabaseCmd, log *logrus.Entry) *Listener {
	return &Listener{
		config:        config,
		conn:          conn,
		relayconn:     relayconn,
		messages:      messages,
		beefyMessages: beefyMessages,
		log:           log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return li.pollBlocks(ctx)
	})

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	close(li.messages)
	return ctx.Err()
}

func (li *Listener) pollBlocks(ctx context.Context) error {
	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
		return nil
	}

	// Get current block
	block, err := li.conn.api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}
	currentBlock := uint32(block.Number)

	retryInterval := time.Duration(10) * time.Second
	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		default:

			li.log.WithField("block", currentBlock).Debug("Processing block")

			// Get block hash
			finalizedHash, err := li.conn.api.RPC.Chain.GetFinalizedHead()
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch finalized head")
				sleep(ctx, retryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := li.conn.api.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch header for finalized head")
				sleep(ctx, retryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint32(finalizedHeader.Number) {
				li.log.WithFields(logrus.Fields{
					"block":  currentBlock,
					"latest": finalizedHeader.Number,
				}).Trace("Block not yet finalized")
				sleep(ctx, retryInterval)
				continue
			}

			digestItem, err := getAuxiliaryDigestItem(finalizedHeader.Digest)
			if err != nil {
				return err
			}

			if digestItem != nil && digestItem.IsCommitment {
				li.log.WithFields(logrus.Fields{
					"block":          finalizedHeader.Number,
					"channelID":      digestItem.AsCommitment.ChannelID,
					"commitmentHash": digestItem.AsCommitment.Hash.Hex(),
				}).Debug("Found commitment hash in header digest")

				storageKey, err := MakeStorageKey(digestItem.AsCommitment.ChannelID, digestItem.AsCommitment.Hash)
				if err != nil {
					return err
				}

				data, err := li.conn.api.RPC.Offchain.LocalStorageGet(rpcOffchain.Persistent, storageKey)
				if err != nil {
					li.log.WithError(err).Error("Failed to read commitment from offchain storage")
					sleep(ctx, retryInterval)
					continue
				}

				if data != nil {
					li.log.WithFields(logrus.Fields{
						"block":               finalizedHeader.Number,
						"commitmentSizeBytes": len(*data),
					}).Debug("Retrieved commitment from offchain storage")
				} else {
					li.log.WithError(err).Error("Commitment not found in offchain storage")
					continue
				}

				var messages []chainTypes.CommitmentMessage

				err = types.DecodeFromBytes(*data, &messages)
				if err != nil {
					li.log.WithError(err).Error("Faild to decode commitment messages")
				}

				message := chain.SubstrateOutboundMessage{
					ChannelID:      digestItem.AsCommitment.ChannelID,
					CommitmentHash: digestItem.AsCommitment.Hash,
					Commitment:     messages,
				}

				li.messages <- []chain.Message{message}
			}

			currentBlock++
		}
	}
}

func (li *Listener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.relayconn.api.Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
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
				li.log.WithError(err).Error("failed to decode beefy commitment messages")
			}

			li.log.Info("Relaychain Listener witnessed a new Beefy commitment: \n", msg.(string))
			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
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

			beefy := store.NewBeefy(beefyAuthorities, *signedCommitment,
				store.CommitmentWitnessed, common.Hash{}, 0, common.Hash{},
			)
			item, err := beefy.ToItem()
			if err != nil {
				li.log.Error(err)
				continue
			}

			li.log.Info("1: Writing BeefyItem to database with status 'WitnessedCommitment'")
			cmd := store.NewDatabaseCmd(&item, store.Create, nil)
			li.beefyMessages <- cmd
		}
	}
}

type Authorities = [][33]uint8

func (li *Listener) getBeefyAuthorities(blockNumber uint64) (Authorities, error) {
	blockHash, err := li.relayconn.api.RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return Authorities{}, err
	}

	meta, err := li.relayconn.api.RPC.State.GetMetadataLatest()
	if err != nil {
		return Authorities{}, err
	}

	storageKey, err := types.CreateStorageKey(meta, "Beefy", "Authorities", nil, nil)
	if err != nil {
		return Authorities{}, err
	}

	storageChangeSet, err := li.relayconn.api.RPC.State.QueryStorage([]types.StorageKey{storageKey}, blockHash, blockHash)
	if err != nil {
		return Authorities{}, err
	}

	authorities := Authorities{}
	for _, storageChange := range storageChangeSet {
		for _, keyValueOption := range storageChange.Changes {
			bz, err := keyValueOption.MarshalJSON()
			if err != nil {
				return Authorities{}, err
			}

			err = types.DecodeFromBytes(bz, &authorities)
			if err != nil {
				return Authorities{}, err
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

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

func getAuxiliaryDigestItem(digest types.Digest) (*chainTypes.AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			return &auxDigestItem, nil
		}
	}
	return nil, nil
}
