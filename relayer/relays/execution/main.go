package execution

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"sort"
	"time"

	"github.com/snowfork/snowbridge/relayer/ofac"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config          *Config
	keypair         *sr25519.Keypair
	paraconn        *parachain.Connection
	ethconn         *ethereum.Connection
	gatewayContract *contracts.Gateway
	beaconHeader    *header.Header
	writer          *parachain.ParachainWriter
	headerCache     *ethereum.HeaderCache
	ofac            *ofac.OFAC
	chainID         *big.Int
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	paraconn := parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	ethconn := ethereum.NewConnection(&r.config.Source.Ethereum, nil)

	err := paraconn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return err
	}
	r.paraconn = paraconn

	err = ethconn.Connect(ctx)
	if err != nil {
		return err
	}
	r.ethconn = ethconn

	r.writer = parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = r.writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethconn},
	)
	if err != nil {
		return err
	}
	r.headerCache = headerCache

	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, ethconn.Client())
	if err != nil {
		return err
	}
	r.gatewayContract = contract

	p := protocol.New(r.config.Source.Beacon.Spec, r.config.Sink.Parachain.HeaderRedundancy)

	r.ofac = ofac.New(r.config.OFAC.Enabled, r.config.OFAC.ApiKey)

	store := store.New(r.config.Source.Beacon.DataStore.Location, r.config.Source.Beacon.DataStore.MaxEntries, *p)
	store.Connect()

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.Endpoint, r.config.Source.Beacon.StateEndpoint)
	beaconHeader := header.New(
		r.writer,
		beaconAPI,
		r.config.Source.Beacon.Spec,
		&store,
		p,
		0, // setting is not used in the execution relay
	)
	r.beaconHeader = &beaconHeader

	r.chainID, err = r.ethconn.Client().NetworkID(ctx)
	if err != nil {
		return err
	}

	log.WithFields(log.Fields{
		"relayerId":     r.config.Schedule.ID,
		"relayerCount":  r.config.Schedule.TotalRelayerCount,
		"sleepInterval": r.config.Schedule.SleepInterval,
		"chainId":       r.chainID,
	}).Info("relayer config")

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-time.After(60 * time.Second):
			log.WithFields(log.Fields{
				"channelId": r.config.Source.ChannelID,
			}).Info("Polling Nonces")

			paraNonce, err := r.fetchLatestParachainNonce()
			if err != nil {
				return err
			}

			ethNonce, err := r.fetchEthereumNonce(ctx)
			if err != nil {
				return err
			}

			log.WithFields(log.Fields{
				"channelId":           types.H256(r.config.Source.ChannelID).Hex(),
				"paraNonce":           paraNonce,
				"ethNonce":            ethNonce,
				"instantVerification": r.config.InstantVerification,
			}).Info("Polled Nonces")

			if paraNonce == ethNonce {
				continue
			}

			blockNumber, err := ethconn.Client().BlockNumber(ctx)
			if err != nil {
				return fmt.Errorf("get last block number: %w", err)
			}

			events, err := r.findEvents(ctx, blockNumber, paraNonce+1)
			if err != nil {
				return fmt.Errorf("find events: %w", err)
			}

			for _, ev := range events {
				err := r.waitAndSend(ctx, ev)
				if errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
					log.WithField("nonce", ev.Nonce).Info("beacon header not finalized yet")
					continue
				} else if err != nil {
					return fmt.Errorf("submit event: %w", err)
				}
			}
		}
	}
}

func (r *Relay) writeToParachain(ctx context.Context, proof scale.ProofPayload, inboundMsg *parachain.Message) error {
	inboundMsg.Proof.ExecutionProof = proof.HeaderPayload

	log.WithFields(logrus.Fields{
		"EventLog": inboundMsg.EventLog,
		"Proof":    inboundMsg.Proof,
	}).Debug("Generated message from Ethereum log")

	// There is already a valid finalized header on-chain that can prove the message
	if proof.FinalizedPayload == nil {
		err := r.writer.WriteToParachainAndWatch(ctx, "EthereumInboundQueue.submit", inboundMsg)
		if err != nil {
			return fmt.Errorf("submit message to inbound queue: %w", err)
		}

		return nil
	}

	log.WithFields(logrus.Fields{
		"finalized_slot": proof.FinalizedPayload.Payload.FinalizedHeader.Slot,
		"finalized_root": proof.FinalizedPayload.FinalizedHeaderBlockRoot,
		"message_slot":   proof.HeaderPayload.Header.Slot,
	}).Debug("Batching finalized header update with message")

	extrinsics := []string{"EthereumBeaconClient.submit", "EthereumInboundQueue.submit"}
	payloads := []interface{}{proof.FinalizedPayload.Payload, inboundMsg}
	// Batch the finalized header update with the inbound message
	err := r.writer.BatchCall(ctx, extrinsics, payloads)
	if err != nil {
		return fmt.Errorf("batch call containing finalized header update and inbound queue message: %w", err)
	}

	return nil
}

func (r *Relay) fetchLatestParachainNonce() (uint64, error) {
	paraID := r.config.Source.ChannelID
	encodedParaID, err := types.EncodeToBytes(r.config.Source.ChannelID)
	if err != nil {
		return 0, err
	}

	paraNonceKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumInboundQueue", "Nonce", encodedParaID, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for EthereumInboundQueue.Nonce(%v): %w",
			paraID, err)
	}
	var paraNonce uint64
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(paraNonceKey, &paraNonce)
	if err != nil {
		return 0, fmt.Errorf("fetch storage EthereumInboundQueue.Nonce(%v): %w",
			paraID, err)
	}
	if !ok {
		paraNonce = 0
	}

	return paraNonce, nil
}

func (r *Relay) fetchEthereumNonce(ctx context.Context) (uint64, error) {
	opts := bind.CallOpts{
		Context: ctx,
	}
	_, ethOutboundNonce, err := r.gatewayContract.ChannelNoncesOf(&opts, r.config.Source.ChannelID)
	if err != nil {
		return 0, fmt.Errorf("fetch Gateway.ChannelNoncesOf(%v): %w", r.config.Source.ChannelID, err)
	}

	return ethOutboundNonce, nil
}

const BlocksPerQuery = 4096

func (r *Relay) findEvents(
	ctx context.Context,
	latestFinalizedBlockNumber uint64,
	start uint64,
) ([]*contracts.GatewayOutboundMessageAccepted, error) {

	channelID := r.config.Source.ChannelID

	var allEvents []*contracts.GatewayOutboundMessageAccepted

	blockNumber := latestFinalizedBlockNumber

	for {
		var begin uint64
		if blockNumber < BlocksPerQuery {
			begin = 0
		} else {
			begin = blockNumber - BlocksPerQuery
		}

		opts := bind.FilterOpts{
			Start:   begin,
			End:     &blockNumber,
			Context: ctx,
		}

		done, events, err := r.findEventsWithFilter(&opts, channelID, start)
		if err != nil {
			return nil, fmt.Errorf("filter events: %w", err)
		}

		if len(events) > 0 {
			allEvents = append(allEvents, events...)
		}

		blockNumber = begin

		if done || begin == 0 {
			break
		}
	}

	sort.SliceStable(allEvents, func(i, j int) bool {
		return allEvents[i].Nonce < allEvents[j].Nonce
	})

	return allEvents, nil
}

func (r *Relay) findEventsWithFilter(opts *bind.FilterOpts, channelID [32]byte, start uint64) (bool, []*contracts.GatewayOutboundMessageAccepted, error) {
	iter, err := r.gatewayContract.FilterOutboundMessageAccepted(opts, [][32]byte{channelID}, [][32]byte{})
	if err != nil {
		return false, nil, err
	}

	var events []*contracts.GatewayOutboundMessageAccepted
	done := false

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return false, nil, err
			}
			break
		}
		if iter.Event.Nonce >= start {
			events = append(events, iter.Event)
		}
		if iter.Event.Nonce == start && opts.Start != 0 {
			// This iteration of findEventsWithFilter contains the last nonce we are interested in,
			// although the nonces might not be ordered in ascending order in the iterator. So there might be more
			// nonces that need to be appended (and we need to keep looping until "more" is false, even though we
			// already have found the oldest nonce.
			done = true
		}
	}

	if done {
		iter.Close()
	}

	return done, events, nil
}

func (r *Relay) makeInboundMessage(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	event *contracts.GatewayOutboundMessageAccepted,
) (*parachain.Message, error) {
	receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
	if err != nil {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to get receipt trie for event")
		return nil, err
	}

	msg, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
	if err != nil {
		log.WithFields(logrus.Fields{
			"address":     event.Raw.Address.Hex(),
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to generate message from ethereum event")
		return nil, err
	}

	log.WithFields(logrus.Fields{
		"blockHash":   event.Raw.BlockHash.Hex(),
		"blockNumber": event.Raw.BlockNumber,
		"txHash":      event.Raw.TxHash.Hex(),
	}).Info("found message")

	return msg, nil
}

func (r *Relay) waitAndSend(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted) (err error) {
	ethNonce := ev.Nonce
	waitingPeriod := (ethNonce + r.config.Schedule.TotalRelayerCount - r.config.Schedule.ID) % r.config.Schedule.TotalRelayerCount
	log.WithFields(logrus.Fields{
		"waitingPeriod": waitingPeriod,
	}).Info("relayer waiting period")

	var cnt uint64
	for {
		// Check the nonce again in case another relayer processed the message while this relayer downloading beacon state
		isProcessed, err := r.isMessageProcessed(ev.Nonce)
		if err != nil {
			return fmt.Errorf("is message procssed: %w", err)
		}
		// If the message is already processed we shouldn't submit it again
		if isProcessed {
			return nil
		}
		// Check if the beacon header is finalized
		err = r.isInFinalizedBlock(ctx, ev)
		if err != nil {
			return fmt.Errorf("check beacon header finalized: %w", err)
		}
		if cnt == waitingPeriod {
			break
		}
		log.Info(fmt.Sprintf("sleeping for %d seconds.", time.Duration(r.config.Schedule.SleepInterval)))

		time.Sleep(time.Duration(r.config.Schedule.SleepInterval) * time.Second)
		cnt++
	}
	err = r.doSubmit(ctx, ev)
	if err != nil {
		return fmt.Errorf("submit inbound message: %w", err)
	}

	return nil
}

func (r *Relay) doSubmit(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted) error {
	inboundMsg, err := r.makeInboundMessage(ctx, r.headerCache, ev)
	if err != nil {
		return fmt.Errorf("make outgoing message: %w", err)
	}

	logger := log.WithFields(log.Fields{
		"ethNonce":    ev.Nonce,
		"msgNonce":    ev.Nonce,
		"address":     ev.Raw.Address.Hex(),
		"blockHash":   ev.Raw.BlockHash.Hex(),
		"blockNumber": ev.Raw.BlockNumber,
		"txHash":      ev.Raw.TxHash.Hex(),
		"txIndex":     ev.Raw.TxIndex,
		"channelID":   types.H256(ev.ChannelID).Hex(),
	})

	source, err := r.getTransactionSender(ctx, ev)
	if err != nil {
		return err
	}

	destination, err := r.getTransactionDestination(ev)
	if err != nil {
		return err
	}

	banned, err := r.ofac.IsBanned(source, destination)
	if err != nil {
		return err
	}
	if banned {
		log.Fatal("banned address found")
		return errors.New("banned address found")
	} else {
		log.Info("address is not banned, continuing")
	}

	nextBlockNumber := new(big.Int).SetUint64(ev.Raw.BlockNumber + 1)

	blockHeader, err := r.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return fmt.Errorf("get block header: %w", err)
	}

	// ParentBeaconRoot in https://eips.ethereum.org/EIPS/eip-4788 from Deneb onward
	proof, err := r.beaconHeader.FetchExecutionProof(*blockHeader.ParentBeaconRoot, r.config.InstantVerification)
	if errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
		return err
	}
	if err != nil {
		return fmt.Errorf("fetch execution header proof: %w", err)
	}

	// Check the nonce again in case another relayer processed the message while this relayer downloading beacon state
	isProcessed, err := r.isMessageProcessed(ev.Nonce)
	if err != nil {
		return fmt.Errorf("is message processed: %w", err)
	}
	// If the message is already processed we shouldn't submit it again
	if isProcessed {
		return nil
	}

	err = r.writeToParachain(ctx, proof, inboundMsg)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}

	paraNonce, err := r.fetchLatestParachainNonce()
	if err != nil {
		return fmt.Errorf("fetch latest parachain nonce: %w", err)
	}
	if paraNonce != ev.Nonce {
		return fmt.Errorf("inbound message fail to execute")
	}
	logger.Info("inbound message executed successfully")

	return nil
}

// isMessageProcessed checks if the provided event nonce has already been processed on-chain.
func (r *Relay) isMessageProcessed(eventNonce uint64) (bool, error) {
	paraNonce, err := r.fetchLatestParachainNonce()
	if err != nil {
		return false, fmt.Errorf("fetch latest parachain nonce: %w", err)
	}
	// Check the nonce again in case another relayer processed the message while this relayer downloading beacon state
	if eventNonce <= paraNonce {
		log.WithField("nonce", paraNonce).Info("message picked up by another relayer, skipped")
		return true, nil
	}

	return false, nil
}

// isInFinalizedBlock checks if the block containing the event is a finalized block.
func (r *Relay) isInFinalizedBlock(ctx context.Context, event *contracts.GatewayOutboundMessageAccepted) error {
	nextBlockNumber := new(big.Int).SetUint64(event.Raw.BlockNumber + 1)

	blockHeader, err := r.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return fmt.Errorf("get block header: %w", err)
	}

	return r.beaconHeader.CheckHeaderFinalized(*blockHeader.ParentBeaconRoot, r.config.InstantVerification)
}

func (r *Relay) getTransactionSender(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted) (string, error) {
	tx, _, err := r.ethconn.Client().TransactionByHash(ctx, ev.Raw.TxHash)
	if err != nil {
		return "", err
	}

	sender, err := ethtypes.Sender(ethtypes.LatestSignerForChainID(r.chainID), tx)
	if err != nil {
		return "", fmt.Errorf("retrieve message sender: %w", err)
	}

	log.WithFields(log.Fields{
		"sender": sender,
	}).Debug("extracted sender from transaction")

	return sender.Hex(), nil
}

func (r *Relay) getTransactionDestination(ev *contracts.GatewayOutboundMessageAccepted) (string, error) {
	destination, err := parachain.GetDestination(ev.Payload)
	if err != nil {
		return "", fmt.Errorf("fetch execution header proof: %w", err)
	}

	if destination == "" {
		return "", nil
	}

	destinationSS58, err := parachain.SS58Encode(destination, r.config.Sink.SS58Prefix)
	if err != nil {
		return "", fmt.Errorf("ss58 encode: %w", err)
	}

	log.WithFields(log.Fields{
		"destinationSS58": destinationSS58,
		"destination":     destination,
	}).Debug("extracted destination from message")

	return destinationSS58, nil
}
