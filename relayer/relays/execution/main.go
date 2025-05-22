package execution

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"path/filepath"
	"runtime"
	"sort"
	"time"

	"github.com/snowfork/snowbridge/relayer/ofac"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"golang.org/x/sync/errgroup"
)

// ConfigureLogger sets up the logger with colors, emojis, and better formatting
func ConfigureLogger() {
	// Set the formatter to include colors and timestamps
	log.SetFormatter(&logrus.TextFormatter{
		FullTimestamp:   true,
		TimestampFormat: "2006-01-02 15:04:05",
		ForceColors:     true,
		DisableColors:   false,
		CallerPrettyfier: func(f *runtime.Frame) (string, string) {
			// Extract just the filename, not the full path
			filename := filepath.Base(f.File)
			return "", fmt.Sprintf("%s:%d", filename, f.Line)
		},
	})

	// Set the log level to Info by default
	log.SetLevel(logrus.DebugLevel)

	// Add some custom fields that will be included in all log messages
	log.SetReportCaller(true)
}

type Relay struct {
	config          *Config
	keypair         *signature.KeyringPair
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
	keypair *signature.KeyringPair,
) *Relay {
	// Configure the logger with colors and better formatting
	ConfigureLogger()

	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	log.WithFields(log.Fields{
		"parachain_endpoint": r.config.Sink.Parachain.Endpoint,
		"ethereum_endpoint":  r.config.Source.Ethereum.Endpoint,
		"gateway_address":    r.config.Source.Contracts.Gateway,
	}).Info("🚀 Starting execution relay")
	
	paraconn := parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair)
	ethconn := ethereum.NewConnection(&r.config.Source.Ethereum, nil)

	log.Info("🔌 Connecting to substrate chain...")
	err := paraconn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		log.WithError(err).Error("❌ Failed to connect to substrate chain")
		return err
	}
	log.Info("✅ Successfully connected to substrate chain")
	r.paraconn = paraconn

	log.Info("🔌 Connecting to Ethereum...")
	err = ethconn.Connect(ctx)
	if err != nil {
		log.WithError(err).Error("❌ Failed to connect to Ethereum")
		return err
	}
	log.Info("✅ Successfully connected to Ethereum")
	r.ethconn = ethconn

	log.Info("📝 Initializing substrate chain writer...")
	r.writer = parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = r.writer.Start(ctx, eg)
	if err != nil {
		log.WithError(err).Error("❌ Failed to start substrate chain writer")
		return err
	}
	log.Info("✅ Substrate chain writer started successfully")

	log.Info("📦 Creating Ethereum header cache...")
	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethconn},
	)
	if err != nil {
		log.WithError(err).Error("❌ Failed to create header cache")
		return err
	}
	r.headerCache = headerCache
	log.Info("✅ Ethereum header cache created successfully")

	log.WithField("gateway_address", r.config.Source.Contracts.Gateway).Info("🔗 Connecting to Gateway contract...")
	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, ethconn.Client())
	if err != nil {
		log.WithError(err).WithField("address", address.Hex()).Error("❌ Failed to connect to Gateway contract")
		return err
	}
	log.Info("✅ Gateway contract connected successfully")
	r.gatewayContract = contract

	log.Info("⚡ Setting up beacon protocol...")
	p := protocol.New(r.config.Source.Beacon.Spec, r.config.Sink.Parachain.HeaderRedundancy)

	log.WithField("ofac_enabled", r.config.OFAC.Enabled).Info("🔒 Initializing OFAC compliance")
	r.ofac = ofac.New(r.config.OFAC.Enabled, r.config.OFAC.ApiKey)

	log.WithFields(log.Fields{
		"store_location": r.config.Source.Beacon.DataStore.Location,
		"max_entries":    r.config.Source.Beacon.DataStore.MaxEntries,
	}).Info("💾 Connecting to beacon store...")
	store := store.New(r.config.Source.Beacon.DataStore.Location, r.config.Source.Beacon.DataStore.MaxEntries, *p)
	store.Connect()
	log.Info("✅ Beacon store connected successfully")

	log.WithFields(log.Fields{
		"beacon_endpoint": r.config.Source.Beacon.Endpoint,
		"state_endpoint":  r.config.Source.Beacon.StateEndpoint,
	}).Info("🔌 Initializing beacon client...")
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
	log.Info("✅ Beacon header initialized successfully")

	log.Info("🔍 Fetching Ethereum network ID...")
	r.chainID, err = r.ethconn.Client().NetworkID(ctx)
	if err != nil {
		log.WithError(err).Error("❌ Failed to fetch Ethereum network ID")
		return err
	}
	log.WithField("chain_id", r.chainID).Info("✅ Ethereum network ID fetched successfully")

	log.WithFields(log.Fields{
		"relayerId":     r.config.Schedule.ID,
		"relayerCount":  r.config.Schedule.TotalRelayerCount,
		"sleepInterval": r.config.Schedule.SleepInterval,
		"chainId":       r.chainID,
	}).Info("⚙️  Relayer configuration")

	log.Info("🔄 Starting main polling loop...")
	for {
		select {
		case <-ctx.Done():
			log.Info("🛑 Context done, stopping execution relay")
			return nil
		case <-time.After(60 * time.Second):
			log.Info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
			log.Info("🔄 POLL CYCLE START")
			log.Info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

			log.Info("🔍 Polling Ethereum nonce...")
			ethNonce, err := r.fetchEthereumNonce(ctx)
			if err != nil {
				log.WithError(err).Error("❌ Failed to fetch Ethereum nonce")
				return err
			}
			log.WithField("ethNonce", ethNonce).Info("✅ Successfully fetched Ethereum nonce")

			log.Info("🔍 Fetching unprocessed substrate chain nonces...")
			paraNonces, err := r.fetchUnprocessedParachainNonces(ethNonce)
			if err != nil {
				log.WithError(err).Error("❌ Failed to fetch unprocessed substrate chain nonces")
				return err
			}

			log.WithFields(log.Fields{
				"paraNonces":          paraNonces,
				"ethNonce":            ethNonce,
				"nonce_count":         len(paraNonces),
				"instantVerification": r.config.InstantVerification,
			}).Info("📊 Nonce polling results")

			log.Info("🔍 Fetching latest Ethereum block number...")
			blockNumber, err := ethconn.Client().BlockNumber(ctx)
			if err != nil {
				log.WithError(err).Error("❌ Failed to get last block number")
				return fmt.Errorf("get last block number: %w", err)
			}

			log.WithFields(log.Fields{
				"blockNumber": blockNumber,
			}).Info("📦 Current Ethereum block number")

			for _, paraNonce := range paraNonces {
				log.WithFields(log.Fields{
					"nonce": paraNonce,
				}).Info("🔍 Finding events for nonce")
				
				log.WithFields(log.Fields{
					"blockNumber": blockNumber,
					"paraNonce":   paraNonce,
				}).Debug("🔎 Search parameters for finding events")
				
				events, err := r.findEvents(ctx, blockNumber, paraNonce)
				if err != nil {
					log.WithError(err).WithField("nonce", paraNonce).Error("❌ Failed to find events for nonce")
					return fmt.Errorf("find events: %w", err)
				}

				log.WithFields(log.Fields{
					"events_count": len(events),
					"paraNonce":    paraNonce,
				}).Info("📦 Found events for nonce")
				
				if len(events) == 0 {
					log.WithField("nonce", paraNonce).Info("⏭️  No events found for nonce, skipping")
					continue
				}

				for i, ev := range events {
					log.WithFields(log.Fields{
						"event_index":  i,
						"nonce":        ev.Nonce,
						"block_number": ev.Raw.BlockNumber,
						"tx_hash":      ev.Raw.TxHash.Hex(),
					}).Info("⚡ Processing event")
					
					err := r.waitAndSend(ctx, ev)
					if errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
						log.WithField("nonce", ev.Nonce).Info("⏳ Beacon header not finalized yet, will retry in next cycle")
						continue
					} else if err != nil {
						log.WithError(err).WithField("nonce", ev.Nonce).Error("❌ Failed to submit event")
						return fmt.Errorf("submit event: %w", err)
					}
					
					log.WithFields(log.Fields{
						"nonce":    ev.Nonce,
						"tx_hash":  ev.Raw.TxHash.Hex(),
					}).Info("✅ Successfully processed event")
				}
			}
			
			log.Info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
			log.Info("✅ POLL CYCLE COMPLETE")
			log.Info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
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
		// Create detailed logs about the message structure
		log.WithFields(logrus.Fields{
			"message_type": "InboundQueueV2.submit",
			"event_log_address": inboundMsg.EventLog.Address.Hex(),
			"event_log_topics_count": len(inboundMsg.EventLog.Topics),
			"event_log_data_length": len(inboundMsg.EventLog.Data),
			"execution_proof_header_slot": inboundMsg.Proof.ExecutionProof.Header.Slot,
			"execution_proof_header_parent_root": inboundMsg.Proof.ExecutionProof.Header.ParentRoot.Hex(),
			"execution_proof_header_state_root": inboundMsg.Proof.ExecutionProof.Header.StateRoot.Hex(),
			"execution_branch_length": len(inboundMsg.Proof.ExecutionProof.ExecutionBranch),
			"receipt_proof_keys_length": len(inboundMsg.Proof.ReceiptProof.Keys),
			"receipt_proof_values_length": len(inboundMsg.Proof.ReceiptProof.Values),
		}).Info("📤 Submitting message to parachain without finalized header update")
		
		err := r.writer.WriteToParachainAndWatch(ctx, "InboundQueueV2.submit", inboundMsg)
		if err != nil {
			log.WithFields(logrus.Fields{
				"error": err.Error(),
				"message_type": "InboundQueueV2.submit",
				"event_log_address": inboundMsg.EventLog.Address.Hex(),
				"event_log_topics_count": len(inboundMsg.EventLog.Topics),
				"event_log_data_length": len(inboundMsg.EventLog.Data),
				"execution_proof_header_slot": inboundMsg.Proof.ExecutionProof.Header.Slot,
			}).Error("❌ Failed to submit message to inbound queue")
			return fmt.Errorf("submit message to inbound queue: %w", err)
		}

		return nil
	}

	log.WithFields(logrus.Fields{
		"finalized_slot": proof.FinalizedPayload.Payload.FinalizedHeader.Slot,
		"finalized_root": proof.FinalizedPayload.FinalizedHeaderBlockRoot,
		"message_slot":   proof.HeaderPayload.Header.Slot,
		"execution_proof_header": proof.HeaderPayload.Header.ToJSON(),
		"execution_branch_length": len(proof.HeaderPayload.ExecutionBranch),
	}).Info("📤 Batching finalized header update with message")

	extrinsics := []string{"EthereumBeaconClient.submit", "InboundQueueV2.submit"}
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
			"InboundQueueV2",
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
	bucketKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "InboundQueueV2", "NonceBitmap", encodedBucket)
	if err != nil {
		return false, fmt.Errorf("create storage key for InboundQueueV2.NonceBitmap: %w", err)
	}

	var bucketValue types.U128
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(bucketKey, &bucketValue)

	if err != nil {
		return false, fmt.Errorf("fetch storage InboundQueueV2.NonceBitmap keys: %w", err)
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

	logger.WithFields(log.Fields{
		"event_log_address": inboundMsg.EventLog.Address.Hex(),
		"event_log_topics_count": len(inboundMsg.EventLog.Topics),
		"event_log_data_length": len(inboundMsg.EventLog.Data),
		"event_log_first_topic": inboundMsg.EventLog.Topics[0].Hex(),
	}).Info("🔍 Created inbound message from Ethereum event")

	nextBlockNumber := new(big.Int).SetUint64(ev.Raw.BlockNumber + 1)

	blockHeader, err := r.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return fmt.Errorf("get block header: %w", err)
	}

	logger.WithFields(log.Fields{
		"next_block_number": nextBlockNumber.Uint64(),
		"parent_beacon_root": blockHeader.ParentBeaconRoot.Hex(),
	}).Info("🔍 Fetching execution proof for block header")

	// ParentBeaconRoot in https://eips.ethereum.org/EIPS/eip-4788 from Deneb onward
	proof, err := r.beaconHeader.FetchExecutionProof(*blockHeader.ParentBeaconRoot, r.config.InstantVerification)
	if errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
		return err
	}
	if err != nil {
		return fmt.Errorf("fetch execution header proof: %w", err)
	}

	logger.WithFields(log.Fields{
		"header_slot": proof.HeaderPayload.Header.Slot,
		"has_finalized_payload": proof.FinalizedPayload != nil,
		"ancestry_proof_has_value": proof.HeaderPayload.AncestryProof.HasValue,
		"execution_branch_length": len(proof.HeaderPayload.ExecutionBranch),
	}).Info("✅ Successfully fetched execution proof")

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
