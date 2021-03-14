// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/parachain"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	config   *Config
	conn     *Connection
	econn    *ethereum.Connection
	contract *lightclientbridge.Contract
	messages chan<- []chain.Message
	beefy    chan parachain.BeefyCommitmentInfo
	log      *logrus.Entry
}

func NewListener(config *Config, conn *Connection, econn *ethereum.Connection, messages chan<- []chain.Message,
	beefy chan parachain.BeefyCommitmentInfo, log *logrus.Entry) *Listener {
	return &Listener{
		config:   config,
		conn:     conn,
		econn:    econn,
		messages: messages,
		beefy:    beefy,
		log:      log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {
	contract, err := lightclientbridge.NewContract(common.HexToAddress(li.config.Ethereum.Contracts.RelayBridgeLightClient), li.econn.GetClient())
	if err != nil {
		return err
	}
	li.contract = contract

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	eg.Go(func() error {
		return li.pollEthereumBlocks(ctx)
	})

	eg.Go(func() error {
		return li.pollLightBridgeEvents(ctx)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	close(li.messages)
	return ctx.Err()
}

func (li *Listener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.conn.api.Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	received := 0
	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case msg := <-ch:

			signedCommitment := &parachain.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Failed to decode beefy commitment messages")
			}

			received++
			li.log.Info("--------------------------------------------------------------")
			li.log.Info("Witnessed new BEEFY commitment: ", received)

			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			// Construct BEEFY merkle tree
			beefyValidatorAddresses := []common.Address{
				common.HexToAddress("0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b"),
				common.HexToAddress("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"),
			}

			// TODO: query beefy authorities
			// var output interface{}
			// var blockNumber *uint64
			// if blockNumber == nil {
			// 	err = li.conn.api.Client.Call(&output, "beefy_getAuthorities")
			// } else {
			// 	err = li.conn.api.Client.Call(&output, "beefy_Authorities", *blockNumber)
			// }
			// if err != nil {
			// 	panic(err)
			// }

			// JavaScript query beefy authorities:
			// async function getAuthoritiesDirect(api) {
			// // For some reason the polkadot-js beefy.authorities function is not returning enough bytes.
			// // This function just manually gets them.
			// const beefyStorageQuery = "0x08c41974a97dbf15cfbec28365bea2da5e0621c4869aa60c02be9adcc98a0d1d";
			// const authorities = await api.rpc.state.getStorage(beefyStorageQuery);
			// return authorities;
			// }

			beefyCommitmentInfo := parachain.NewBeefyCommitmentInfo(beefyValidatorAddresses, signedCommitment)

			li.messages <- []chain.Message{beefyCommitmentInfo}
		}
	}
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

// pollEthereumBlocks transitions BEEFY commitments from InitialVerificationTxConfirmed to ReadyToComplete status
func (li *Listener) pollEthereumBlocks(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)
	_, headerCtx := errgroup.WithContext(ctx)
	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case <-headerCtx.Done():
			return li.onDone(ctx)
		case gethheader := <-headers:
			blockNumber := gethheader.Number.Uint64()
			for beefyCommitment := range li.beefy {
				if beefyCommitment.Status == parachain.InitialVerificationTxConfirmed {
					if beefyCommitment.CompleteOnBlock >= blockNumber {
						beefyCommitment.Status = parachain.ReadyToComplete
						li.messages <- []chain.Message{beefyCommitment}
					}
				}
			}
		}
	}
}

// pollLightBridgeEvents fetches events from the LightClientBridge every block
func (li *Listener) pollLightBridgeEvents(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)
	_, headerCtx := errgroup.WithContext(ctx)

	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case <-headerCtx.Done():
			return li.onDone(ctx)
		case gethheader := <-headers:
			if li.beefy == nil {
				li.log.Info("Not polling block details since channel is nil") // TODO: 'continue' here?
			}

			blockNumber := gethheader.Number.Uint64()

			// Query ContractInitialVerificationSuccessful events
			var events []*lightclientbridge.ContractInitialVerificationSuccessful
			contractEvents, err := li.queryEvents(ctx, li.contract, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
			}
			events = append(events, contractEvents...)

			li.processEvents(ctx, events)
		}
	}
}

// queryEvents queries ContractInitialVerificationSuccessful events from the LightClientBridge contract
func (li *Listener) queryEvents(ctx context.Context, contract *lightclientbridge.Contract, start uint64,
	end *uint64) ([]*lightclientbridge.ContractInitialVerificationSuccessful, error) {
	var events []*lightclientbridge.ContractInitialVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := contract.FilterInitialVerificationSuccessful(&filterOps)
	if err != nil {
		return nil, err
	}

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return nil, err
			}
			break
		}

		events = append(events, iter.Event)
	}

	return events, nil
}

// processEvents matches events to BEEFY commitment info by transaction hash
func (li *Listener) processEvents(ctx context.Context, events []*lightclientbridge.ContractInitialVerificationSuccessful) {
	for _, event := range events {
		for beefyCommitment := range li.beefy {
			if beefyCommitment.Status == parachain.InitialVerificationTxSent {
				if beefyCommitment.InitialVerificationTxHash.Hex() == event.Raw.TxHash.Hex() {
					beefyCommitment.Status = parachain.InitialVerificationTxConfirmed
					beefyCommitment.CompleteOnBlock = event.Raw.BlockNumber + li.config.Ethereum.BeefyBlockDelay
				}
			}
			li.beefy <- beefyCommitment // TODO: do we need any additional event info?
		}
	}
}
