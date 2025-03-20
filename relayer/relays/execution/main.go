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

			ethNonce, err := r.fetchEthereumNonce(ctx)
			if err != nil {
				return err
			}

			paraNonces, err := r.fetchUnprocessedParachainNonces(ethNonce)
			if err != nil {
				return err
			}

			log.WithFields(log.Fields{
				"channelId":           types.H256(r.config.Source.ChannelID).Hex(),
				"paraNonces":          paraNonces,
				"ethNonce":            ethNonce,
				"instantVerification": r.config.InstantVerification,
			}).Info("Polled Nonces")

			blockNumber, err := ethconn.Client().BlockNumber(ctx)
			if err != nil {
				return fmt.Errorf("get last block number: %w", err)
			}

			log.WithFields(log.Fields{
				"blockNumber": blockNumber,
			}).Info("block number is")

			for _, paraNonce := range paraNonces {
				log.WithFields(log.Fields{
					"nonce": paraNonce,
				}).Info("Finding events for nonce")
				events, err := r.findEvents(ctx, blockNumber, paraNonce)
				if err != nil {
					return fmt.Errorf("find events: %w", err)
				}

				log.WithFields(log.Fields{
					"events":    events,
					"paraNonce": paraNonce,
				}).Info("Found events for nonce")

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
}

func (r *Relay) writeToParachain(ctx context.Context, proof scale.ProofPayload, inboundMsg *parachain.Message) error {
	inboundMsg.Proof.ExecutionProof = proof.HeaderPayload

	log.WithFields(logrus.Fields{
		"EventLog": inboundMsg.EventLog,
		"Proof":    inboundMsg.Proof,
	}).Debug("Generated message from Ethereum log")

	// There is already a valid finalized header on-chain that can prove the message
	if proof.FinalizedPayload == nil {
		err := r.writer.WriteToParachainAndWatch(ctx, "EthereumInboundQueueV2.submit", inboundMsg)
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

	extrinsics := []string{"EthereumBeaconClient.submit", "EthereumInboundQueueV2.submit"}
	payloads := []interface{}{proof.FinalizedPayload.Payload, inboundMsg}
	// Batch the finalized header update with the inbound message
	err := r.writer.BatchCall(ctx, extrinsics, payloads)
	if err != nil {
		return fmt.Errorf("batch call containing finalized header update and inbound queue message: %w", err)
	}

	return nil
}

func (r *Relay) fetchUnprocessedParachainNonces(latest uint64) ([]uint64, error) {
	unprocessedNonces := []uint64{}
	latestBucket := latest / 128

	for b := uint64(0); b <= latestBucket; b++ {
		encodedBucket, err := types.EncodeToBytes(types.NewU128(*big.NewInt(int64(b))))
		bucketKey, _ := types.CreateStorageKey(
			r.paraconn.Metadata(),
			"EthereumInboundQueueV2",
			"NonceBitmap",
			encodedBucket,
			nil,
		)

		var value types.U128
		ok, err := r.paraconn.API().RPC.State.GetStorageLatest(bucketKey, &value)
		if err != nil {
			return nil, fmt.Errorf("failed to read bucket %d: %w", b, err)
		}

		// "Missing" means the chain doesn't store it => it's 0
		if !ok {
			value = types.NewU128(*big.NewInt(0))
		}

		// Now parse bits from value...
		bucketNonces := extractUnprocessedNonces(value, latest, b)
		unprocessedNonces = append(unprocessedNonces, bucketNonces...)
	}

	log.WithFields(logrus.Fields{
		"nonces": unprocessedNonces,
	}).Debug("nonces to be processed")
	return unprocessedNonces, nil
}

func (r *Relay) isParachainNonceSet(index uint64) (bool, error) {
	log.WithFields(logrus.Fields{
		"index": index,
	}).Debug("is parachain nonce set")
	// Calculate the bucket and bit position
	bucket := index / 128
	bitPosition := index % 128

	encodedBucket, err := types.EncodeToBytes(types.NewU128(*big.NewInt(int64(bucket))))
	bucketKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumInboundQueueV2", "NonceBitmap", encodedBucket)
	if err != nil {
		return false, fmt.Errorf("create storage key for EthereumInboundQueueV2.NonceBitmap: %w", err)
	}

	var bucketValue types.U128
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(bucketKey, &bucketValue)

	if err != nil {
		return false, fmt.Errorf("fetch storage EthereumInboundQueueV2.NonceBitmap keys: %w", err)
	}
	if !ok {
		return false, fmt.Errorf("bucket does not exist: %w", err)
	}

	return checkBitState(bucketValue, bitPosition), nil
}

func checkBitState(bucketValue types.U128, bitPosition uint64) bool {
	log.WithFields(logrus.Fields{
		"bucketValue": bucketValue,
		"bitPosition": bitPosition,
	}).Debug("checking bit state")
	mask := new(big.Int).Lsh(big.NewInt(1), uint(bitPosition)) // Create mask for the bit position
	result := new(big.Int).And(bucketValue.Int, mask).Cmp(big.NewInt(0)) != 0
	log.WithFields(logrus.Fields{
		"result":      result,
		"bitPosition": bitPosition,
	}).Debug("check bit state result")
	return result
}

func extractUnprocessedNonces(bitmap types.U128, latest uint64, bucketIndex uint64) []uint64 {
	var unprocessed []uint64
	// Each bucket covers 128 nonces
	baseNonce := bucketIndex * 128

	for i := 0; i < 128; i++ {
		nonce := baseNonce + uint64(i)
		// Ignore nonce 0 since valid nonces start at 1
		if nonce < 1 {
			continue
		}
		// If we've passed the latest nonce to consider, stop checking further bits.
		if nonce > latest {
			break
		}
		// Check if bit `i` is unset (meaning unprocessed).
		mask := new(big.Int).Lsh(big.NewInt(1), uint(i))
		if new(big.Int).And(bitmap.Int, mask).Cmp(big.NewInt(0)) == 0 {
			unprocessed = append(unprocessed, nonce)
		}
	}

	return unprocessed
}

func (r *Relay) fetchEthereumNonce(ctx context.Context) (uint64, error) {
	opts := bind.CallOpts{
		Context: ctx,
	}
	ethOutboundNonce, err := r.gatewayContract.V2OutboundNonce(&opts)
	if err != nil {
		return 0, fmt.Errorf("fetch Gateway.OutboundNonce: %w", err)
	}

	return ethOutboundNonce, nil
}

const BlocksPerQuery = 4096

func (r *Relay) findEvents(
	ctx context.Context,
	latestFinalizedBlockNumber uint64,
	start uint64,
) ([]*contracts.GatewayOutboundMessageAccepted, error) {
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

		done, events, err := r.findEventsWithFilter(&opts, start)
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

func (r *Relay) findEventsWithFilter(opts *bind.FilterOpts, start uint64) (bool, []*contracts.GatewayOutboundMessageAccepted, error) {
	iter, err := r.gatewayContract.FilterOutboundMessageAccepted(opts)
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
	})

	source, err := r.getTransactionSender(ctx, ev)
	if err != nil {
		return err
	}

	banned, err := r.ofac.IsBanned(source, "")
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

	ok, err := r.isParachainNonceSet(ev.Nonce)
	if !ok {
		return fmt.Errorf("inbound message fail to execute")
	}
	logger.Info("inbound message executed successfully")

	return nil
}

// isMessageProcessed checks if the provided event nonce has already been processed on-chain.
func (r *Relay) isMessageProcessed(eventNonce uint64) (bool, error) {
	paraNonces, err := r.fetchUnprocessedParachainNonces(eventNonce)
	if err != nil {
		return false, fmt.Errorf("fetch latest parachain nonce: %w", err)
	}
	// Check the nonce again in case another relayer processed the message while this relayer downloading beacon state

	for _, paraNonce := range paraNonces {
		if eventNonce == paraNonce {
			return false, nil
		}
	}

	return true, nil
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

func (r *Relay) UnprocessedNonces() {

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
