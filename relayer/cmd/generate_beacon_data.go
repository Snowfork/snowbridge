package cmd

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/cmd/run/execution"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	beaconConf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	beaconjson "github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	executionConf "github.com/snowfork/snowbridge/relayer/relays/execution"

	"github.com/cbroglie/mustache"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func generateBeaconFixtureCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-fixture",
		Short: "Generate beacon fixture.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconTestFixture,
	}

	cmd.Flags().String("config", "/tmp/snowbridge/beacon-relay.json", "Path to the beacon relay config")
	cmd.Flags().Bool("wait_until_next_period", true, "Waiting until next period")
	cmd.Flags().Uint32("nonce", 1, "Nonce of the inbound message")
	return cmd
}

func generateBeaconCheckpointCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-checkpoint",
		Short: "Generate beacon checkpoint.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconCheckpoint,
	}

	cmd.Flags().String("config", "/tmp/snowbridge/beacon-relay.json", "Path to the beacon relay config")
	cmd.Flags().Uint64("finalized-slot", 0, "Optional finalized slot to create checkpoint at")
	cmd.Flags().Bool("export-json", false, "Export Json")

	return cmd
}

func generateExecutionUpdateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-execution-update",
		Short: "Generate execution update.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateExecutionUpdate,
	}

	cmd.Flags().String("config", "/tmp/snowbridge/beacon-relay.json", "Path to the beacon relay config")
	cmd.Flags().Uint32("slot", 1, "slot number")
	return cmd
}

func generateInboundFixtureCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-inbound-fixture",
		Short: "Generate inbound fixture.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateInboundFixture,
	}

	cmd.Flags().String("beacon-config", "/tmp/snowbridge/beacon-relay.json", "Path to the beacon relay config")
	cmd.Flags().String("execution-config", "/tmp/snowbridge/execution-relay-asset-hub-0.json", "Path to the beacon relay config")
	cmd.Flags().Uint32("nonce", 1, "Nonce of the inbound message")
	cmd.Flags().String("test_case", "register_token", "Inbound test case")
	return cmd
}

type Data struct {
	CheckpointUpdate      beaconjson.CheckPoint
	SyncCommitteeUpdate   beaconjson.Update
	FinalizedHeaderUpdate beaconjson.Update
	HeaderUpdate          beaconjson.HeaderUpdate
	InboundMessage        parachain.MessageJSON
	TestCase              string
}

type InboundFixture struct {
	FinalizedHeaderUpdate beaconjson.Update     `json:"update"`
	Message               parachain.MessageJSON `json:"message"`
}

const (
	pathToBeaconTestFixtureFiles              = "polkadot-sdk/bridges/snowbridge/pallets/ethereum-client/tests/fixtures"
	pathToInboundQueueFixtureTemplate         = "relayer/templates/beacon-fixtures.mustache"
	pathToInboundQueueFixtureData             = "polkadot-sdk/bridges/snowbridge/pallets/ethereum-client/fixtures/src/lib.rs"
	pathToInboundQueueFixtureTestCaseTemplate = "relayer/templates/inbound-fixtures.mustache"
	pathToInboundQueueFixtureTestCaseData     = "polkadot-sdk/bridges/snowbridge/pallets/inbound-queue/fixtures/src/%s.rs"
)

// Only print the hex encoded call as output of this command
func generateBeaconCheckpoint(cmd *cobra.Command, _ []string) error {
	err := func() error {
		config, err := cmd.Flags().GetString("config")
		if err != nil {
			return err
		}
		finalizedSlot, _ := cmd.Flags().GetUint64("finalized-slot")

		viper.SetConfigFile(config)

		if err := viper.ReadInConfig(); err != nil {
			return err
		}

		var conf beaconConf.Config
		err = viper.Unmarshal(&conf)
		if err != nil {
			return err
		}

		p := protocol.New(conf.Source.Beacon.Spec, conf.Sink.Parachain.HeaderRedundancy)
		store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries, *p)
		store.Connect()
		defer store.Close()

		client := api.NewBeaconClient(conf.Source.Beacon.Endpoint, conf.Source.Beacon.StateEndpoint)
		s := syncer.New(client, &store, p)

		var checkPointScale scale.BeaconCheckpoint
		if finalizedSlot == 0 {
			checkPointScale, err = s.GetCheckpoint()
		} else {
			checkPointScale, err = s.GetCheckpointAtSlot(finalizedSlot)
		}

		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}
		exportJson, err := cmd.Flags().GetBool("export-json")
		if err != nil {
			return fmt.Errorf("get export-json flag: %w", err)
		}
		if exportJson {
			initialSync := checkPointScale.ToJSON()
			err = writeJSONToFile(initialSync, "dump-initial-checkpoint.json")
			if err != nil {
				return fmt.Errorf("write initial sync to file: %w", err)
			}
		}
		checkPointCallBytes, _ := types.EncodeToBytes(checkPointScale)
		checkPointCallHex := hex.EncodeToString(checkPointCallBytes)
		fmt.Println(checkPointCallHex)
		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon checkpoint")
	}

	return nil
}

func generateBeaconTestFixture(cmd *cobra.Command, _ []string) error {
	err := func() error {
		ctx := context.Background()

		config, err := cmd.Flags().GetString("config")
		if err != nil {
			return err
		}

		viper.SetConfigFile(config)
		if err = viper.ReadInConfig(); err != nil {
			return err
		}

		var conf beaconConf.Config
		err = viper.Unmarshal(&conf)
		if err != nil {
			return err
		}

		p := protocol.New(conf.Source.Beacon.Spec, conf.Sink.Parachain.HeaderRedundancy)

		store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries, *p)
		err = store.Connect()
		if err != nil {
			return err
		}
		defer store.Close()

		log.WithFields(log.Fields{"endpoint": conf.Source.Beacon.Endpoint}).Info("connecting to beacon API")
		client := api.NewBeaconClient(conf.Source.Beacon.Endpoint, conf.Source.Beacon.StateEndpoint)
		s := syncer.New(client, &store, p)

		viper.SetConfigFile("/tmp/snowbridge/execution-relay-asset-hub-0.json")

		if err = viper.ReadInConfig(); err != nil {
			return err
		}

		var executionConfig executionConf.Config
		err = viper.Unmarshal(&executionConfig, viper.DecodeHook(execution.HexHookFunc()))
		if err != nil {
			return fmt.Errorf("unable to parse execution relay config: %w", err)
		}

		ethconn := ethereum.NewConnection(&executionConfig.Source.Ethereum, nil)
		err = ethconn.Connect(ctx)
		if err != nil {
			return err
		}

		headerCache, err := ethereum.NewHeaderBlockCache(
			&ethereum.DefaultBlockLoader{Conn: ethconn},
		)
		if err != nil {
			return err
		}

		// generate InitialUpdate
		initialSyncScale, err := s.GetCheckpoint()
		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}
		initialSync := initialSyncScale.ToJSON()
		err = writeJSONToFile(initialSync, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "initial-checkpoint.json"))
		if err != nil {
			return err
		}
		initialSyncHeaderSlot := initialSync.Header.Slot
		initialSyncPeriod := p.ComputeSyncPeriodAtSlot(initialSyncHeaderSlot)
		initialEpoch := p.ComputeEpochAtSlot(initialSyncHeaderSlot)

		// generate SyncCommitteeUpdate for filling the missing NextSyncCommittee in initial checkpoint
		syncCommitteeUpdateScale, err := s.GetSyncCommitteePeriodUpdate(initialSyncPeriod, 0)
		if err != nil {
			return fmt.Errorf("get sync committee update: %w", err)
		}
		syncCommitteeUpdate := syncCommitteeUpdateScale.Payload.ToJSON()
		log.WithFields(log.Fields{
			"epoch":  initialEpoch,
			"period": initialSyncPeriod,
		}).Info("created initial sync file")
		err = writeJSONToFile(syncCommitteeUpdate, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "sync-committee-update.json"))
		if err != nil {
			return err
		}
		log.Info("created sync committee update file")

		// get inbound message data start
		channelID := executionConfig.Source.ChannelID
		address := common.HexToAddress(executionConfig.Source.Contracts.Gateway)
		gatewayContract, err := contracts.NewGateway(address, ethconn.Client())
		if err != nil {
			return err
		}
		nonce, err := cmd.Flags().GetUint32("nonce")
		if err != nil {
			return err
		}
		event, err := getEthereumEvent(ctx, gatewayContract, channelID, nonce)
		if err != nil {
			return err
		}
		receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
		if err != nil {
			return err
		}
		inboundMessage, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
		if err != nil {
			return err
		}
		messageBlockNumber := event.Raw.BlockNumber

		log.WithFields(log.Fields{
			"message":     inboundMessage,
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": messageBlockNumber,
		}).Info("event is at block")

		finalizedUpdateAfterMessage, err := getFinalizedUpdate(*s, messageBlockNumber)
		if err != nil {
			return err
		}

		finalizedHeaderSlot := uint64(finalizedUpdateAfterMessage.Payload.FinalizedHeader.Slot)

		beaconBlock, blockNumber, err := getBeaconBlockContainingExecutionHeader(*s, messageBlockNumber, finalizedHeaderSlot)
		if err != nil {
			return fmt.Errorf("get beacon block containing header: %w", err)
		}

		beaconBlockSlot, err := strconv.ParseUint(beaconBlock.Data.Message.Slot, 10, 64)
		if err != nil {
			return err
		}

		if blockNumber == messageBlockNumber {
			log.WithFields(log.Fields{
				"slot":        beaconBlock.Data.Message.Slot,
				"blockHash":   beaconBlock.Data.Message.Body.ExecutionPayload.BlockHash,
				"blockNumber": blockNumber,
			}).WithError(err).Info("found execution header containing event")
		}

		checkPoint := cache.Proof{
			FinalizedBlockRoot: finalizedUpdateAfterMessage.FinalizedHeaderBlockRoot,
			BlockRootsTree:     finalizedUpdateAfterMessage.BlockRootsTree,
			Slot:               uint64(finalizedUpdateAfterMessage.Payload.FinalizedHeader.Slot),
		}
		headerUpdateScale, err := s.GetHeaderUpdateBySlotWithCheckpoint(beaconBlockSlot, &checkPoint)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		inboundMessage.Proof.ExecutionProof = headerUpdateScale
		headerUpdate := headerUpdateScale.ToJSON()

		log.WithField("blockNumber", blockNumber).Info("found beacon block by slot")

		messageJSON := inboundMessage.ToJSON()

		err = writeJSONToFile(headerUpdate, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "execution-proof.json"))
		if err != nil {
			return err
		}
		log.Info("created execution update file")
		err = writeJSONToFile(messageJSON, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "inbound-message.json"))
		if err != nil {
			return err
		}
		log.Info("created inbound message file")
		// get inbound message data end

		finalizedUpdate := finalizedUpdateAfterMessage.Payload.ToJSON()
		if finalizedUpdate.AttestedHeader.Slot <= initialSyncHeaderSlot {
			return fmt.Errorf("AttestedHeader slot should be greater than initialSyncHeaderSlot")
		}
		finalizedEpoch := p.ComputeEpochAtSlot(finalizedUpdate.AttestedHeader.Slot)
		if finalizedEpoch <= initialEpoch {
			return fmt.Errorf("epoch in FinalizedUpdate should be greater than initialEpoch")
		}
		finalizedPeriod := p.ComputeSyncPeriodAtSlot(finalizedUpdate.FinalizedHeader.Slot)
		if initialSyncPeriod != finalizedPeriod {
			return fmt.Errorf("initialSyncPeriod should be consistent with finalizedUpdatePeriod")
		}
		err = writeJSONToFile(finalizedUpdate, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "finalized-header-update.json"))
		if err != nil {
			return err
		}
		log.WithFields(log.Fields{
			"epoch":  finalizedEpoch,
			"period": finalizedPeriod,
		}).Info("created finalized header update file")

		// Generate benchmark fixture and inbound fixture
		// Rust file hexes require the 0x of hashes to be removed
		initialSync.RemoveLeadingZeroHashes()
		syncCommitteeUpdate.RemoveLeadingZeroHashes()
		finalizedUpdate.RemoveLeadingZeroHashes()
		headerUpdate.RemoveLeadingZeroHashes()
		messageJSON.RemoveLeadingZeroHashes()

		data := Data{
			CheckpointUpdate:      initialSync,
			SyncCommitteeUpdate:   syncCommitteeUpdate,
			FinalizedHeaderUpdate: finalizedUpdate,
			HeaderUpdate:          headerUpdate,
			InboundMessage:        messageJSON,
		}

		// writing beacon inbound fixtures
		rendered, err := mustache.RenderFile(pathToInboundQueueFixtureTemplate, data)
		if err != nil {
			return fmt.Errorf("render inbound queue benchmark fixture: %w", err)
		}
		log.WithFields(log.Fields{
			"location": pathToInboundQueueFixtureData,
		}).Info("writing result file")
		err = writeRawDataFile(pathToInboundQueueFixtureData, rendered)
		if err != nil {
			return err
		}

		// Generate test fixture in next period (require waiting a long time)
		waitUntilNextPeriod, err := cmd.Flags().GetBool("wait_until_next_period")
		if err != nil {
			return fmt.Errorf("could not parse flag wait_until_next_period: %w", err)
		}
		if waitUntilNextPeriod {
			log.Info("waiting finalized_update in next period (5 hours later), be patient and wait...")
			for {
				nextFinalizedUpdateScale, err := s.GetFinalizedUpdate()
				if err != nil {
					log.Error(err)
					continue
				}
				nextFinalizedUpdate := nextFinalizedUpdateScale.Payload.ToJSON()
				nextFinalizedUpdatePeriod := p.ComputeSyncPeriodAtSlot(nextFinalizedUpdate.FinalizedHeader.Slot)
				if initialSyncPeriod+1 == nextFinalizedUpdatePeriod {
					err := writeJSONToFile(nextFinalizedUpdate, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "next-finalized-header-update.json"))
					if err != nil {
						return err
					}
					log.Info("created next finalized header update file")

					// generate nextSyncCommitteeUpdate
					nextSyncCommitteeUpdateScale, err := s.GetSyncCommitteePeriodUpdate(initialSyncPeriod+1, 0)
					if err != nil {
						log.Error(err)
						continue
					}
					nextSyncCommitteeUpdate := nextSyncCommitteeUpdateScale.Payload.ToJSON()
					err = writeJSONToFile(nextSyncCommitteeUpdate, fmt.Sprintf("%s/%s", pathToBeaconTestFixtureFiles, "next-sync-committee-update.json"))
					if err != nil {
						return err
					}
					log.Info("created next sync committee update file")

					break
				} else {
					log.WithField("slot", nextFinalizedUpdate.FinalizedHeader.Slot).Info("wait 5 minutes for next sync committee period")
					time.Sleep(time.Minute * 5)
				}
			}
		}

		log.Info("done")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon data")
	}

	return nil
}

func writeJSONToFile(data interface{}, path string) error {
	file, _ := json.MarshalIndent(data, "", "  ")

	f, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}

	defer f.Close()

	_, err = f.Write(file)

	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
}

func writeRawDataFile(path string, fileContents string) error {
	f, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}

	defer f.Close()

	_, err = f.Write([]byte(fileContents))

	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
}

func generateExecutionUpdate(cmd *cobra.Command, _ []string) error {
	err := func() error {
		config, err := cmd.Flags().GetString("config")
		if err != nil {
			return err
		}
		beaconSlot, err := cmd.Flags().GetUint32("slot")
		if err != nil {
			return err
		}

		viper.SetConfigFile(config)
		if err := viper.ReadInConfig(); err != nil {
			return err
		}
		var conf beaconConf.Config
		err = viper.Unmarshal(&conf)
		if err != nil {
			return err
		}
		log.WithFields(log.Fields{"endpoint": conf.Source.Beacon.Endpoint}).Info("connecting to beacon API")

		p := protocol.New(conf.Source.Beacon.Spec, conf.Sink.Parachain.HeaderRedundancy)

		store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries, *p)
		store.Connect()
		defer store.Close()

		// generate executionUpdate
		client := api.NewBeaconClient(conf.Source.Beacon.Endpoint, conf.Source.Beacon.StateEndpoint)
		s := syncer.New(client, &store, p)
		blockRoot, err := s.Client.GetBeaconBlockRoot(uint64(beaconSlot))
		if err != nil {
			return fmt.Errorf("fetch block: %w", err)
		}
		headerUpdateScale, err := s.GetHeaderUpdate(blockRoot, nil)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		headerUpdate := headerUpdateScale.ToJSON()
		err = writeJSONToFile(headerUpdate, "tmp/snowbridge/execution-header-update.json")
		if err != nil {
			return err
		}
		log.Info("created execution update file")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon execution update")
	}

	return nil
}

func getEthereumEvent(ctx context.Context, gatewayContract *contracts.Gateway, channelID executionConf.ChannelID, nonce uint32) (*contracts.GatewayOutboundMessageAccepted, error) {
	maxBlockNumber := uint64(10000)

	opts := bind.FilterOpts{
		Start:   1,
		End:     &maxBlockNumber,
		Context: ctx,
	}

	var event *contracts.GatewayOutboundMessageAccepted

	for event == nil {
		log.Info("looking for Ethereum event")

		iter, err := gatewayContract.FilterOutboundMessageAccepted(&opts, [][32]byte{channelID}, [][32]byte{})
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
			if iter.Event.Nonce >= uint64(nonce) {
				event = iter.Event
				iter.Close()
				break
			}
		}

		time.Sleep(5 * time.Second)
	}

	log.WithField("event", event).Info("found event")

	return event, nil
}

func getBeaconBlockContainingExecutionHeader(s syncer.Syncer, messageBlockNumber, finalizedSlot uint64) (api.BeaconBlockResponse, uint64, error) {
	// quick check to see if the blocknumber == slotnumber (often the case in the testnet).
	// in that case we found the beacon block containing the execution header quickly and can return
	beaconBlock, err := s.Client.GetBeaconBlockBySlot(messageBlockNumber)
	if err != nil {
		return api.BeaconBlockResponse{}, 0, err
	}
	blockNumber, err := strconv.ParseUint(beaconBlock.Data.Message.Body.ExecutionPayload.BlockNumber, 10, 64)
	if err != nil {
		return api.BeaconBlockResponse{}, 0, err
	}

	// we've got the block, return it
	if blockNumber == messageBlockNumber {
		log.WithField("blockNumber", blockNumber).Info("found beacon block, same slot as block number")
		return beaconBlock, 0, nil
	}

	log.Info("searching for beacon block by execution block number")

	beaconHeaderSlot := finalizedSlot
	log.WithField("beaconHeaderSlot", beaconHeaderSlot).Info("getting beacon block by slot")

	for blockNumber != messageBlockNumber && beaconHeaderSlot > 1 {
		beaconHeaderSlot = beaconHeaderSlot - 1
		log.WithField("beaconHeaderSlot", beaconHeaderSlot).Info("getting beacon block by slot")

		beaconBlock, blockNumber, err = getBeaconBlockAndBlockNumber(s, beaconHeaderSlot)
		if err != nil {
			return api.BeaconBlockResponse{}, 0, err
		}
	}

	return beaconBlock, blockNumber, nil
}

func getBeaconBlockAndBlockNumber(s syncer.Syncer, slot uint64) (api.BeaconBlockResponse, uint64, error) {
	beaconBlock, err := s.Client.GetBeaconBlockBySlot(slot)
	if err != nil {
		return api.BeaconBlockResponse{}, 0, err
	}
	blockNumber, err := strconv.ParseUint(beaconBlock.Data.Message.Body.ExecutionPayload.BlockNumber, 10, 64)
	if err != nil {
		return api.BeaconBlockResponse{}, 0, err
	}

	log.WithField("blockNumber", blockNumber).Info("found beacon block by slot")

	return beaconBlock, blockNumber, nil
}

func getFinalizedUpdate(s syncer.Syncer, eventBlockNumber uint64) (*scale.Update, error) {
	var blockNumber uint64
	var finalizedUpdate scale.Update
	var err error

	for blockNumber < eventBlockNumber {

		finalizedUpdate, err = s.GetFinalizedUpdate()
		if err != nil {
			return nil, err
		}

		finalizedSlot := uint64(finalizedUpdate.Payload.FinalizedHeader.Slot)
		log.WithField("slot", finalizedSlot).Info("found finalized update at slot")

		beaconBlock, err := s.Client.GetBeaconBlockBySlot(finalizedSlot)
		if err != nil {
			return nil, err
		}

		blockNumber, err = strconv.ParseUint(beaconBlock.Data.Message.Body.ExecutionPayload.BlockNumber, 10, 64)
		if err != nil {
			return nil, err
		}

		if blockNumber > eventBlockNumber {
			log.Info("found finalized block after message")
			break
		}
		// wait for finalized header after event
		log.Info("waiting for chain to finalize after message...")
		time.Sleep(20 * time.Second)
	}

	return &finalizedUpdate, nil
}

func generateInboundFixture(cmd *cobra.Command, _ []string) error {
	err := func() error {
		ctx := context.Background()

		beaconConfig, err := cmd.Flags().GetString("beacon-config")
		if err != nil {
			return err
		}

		executionConfig, err := cmd.Flags().GetString("execution-config")
		if err != nil {
			return err
		}

		viper.SetConfigFile(beaconConfig)
		if err = viper.ReadInConfig(); err != nil {
			return err
		}

		var beaconConf beaconConf.Config
		err = viper.Unmarshal(&beaconConf)
		if err != nil {
			return err
		}

		p := protocol.New(beaconConf.Source.Beacon.Spec, beaconConf.Sink.Parachain.HeaderRedundancy)

		store := store.New(beaconConf.Source.Beacon.DataStore.Location, beaconConf.Source.Beacon.DataStore.MaxEntries, *p)
		store.Connect()
		defer store.Close()

		log.WithFields(log.Fields{"endpoint": beaconConf.Source.Beacon.Endpoint}).Info("connecting to beacon API")
		client := api.NewBeaconClient(beaconConf.Source.Beacon.Endpoint, beaconConf.Source.Beacon.StateEndpoint)
		s := syncer.New(client, &store, p)

		viper.SetConfigFile(executionConfig)

		if err = viper.ReadInConfig(); err != nil {
			return err
		}

		var executionConf executionConf.Config
		err = viper.Unmarshal(&executionConf, viper.DecodeHook(execution.HexHookFunc()))
		if err != nil {
			return fmt.Errorf("unable to parse execution relay config: %w", err)
		}

		ethconn := ethereum.NewConnection(&executionConf.Source.Ethereum, nil)
		err = ethconn.Connect(ctx)
		if err != nil {
			return err
		}

		headerCache, err := ethereum.NewHeaderBlockCache(
			&ethereum.DefaultBlockLoader{Conn: ethconn},
		)
		if err != nil {
			return err
		}

		// get inbound message data start
		channelID := executionConf.Source.ChannelID
		address := common.HexToAddress(executionConf.Source.Contracts.Gateway)
		gatewayContract, err := contracts.NewGateway(address, ethconn.Client())
		if err != nil {
			return err
		}
		nonce, err := cmd.Flags().GetUint32("nonce")
		if err != nil {
			return err
		}
		event, err := getEthereumEvent(ctx, gatewayContract, channelID, nonce)
		if err != nil {
			return err
		}
		receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
		if err != nil {
			return err
		}
		inboundMessage, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
		if err != nil {
			return err
		}
		messageBlockNumber := event.Raw.BlockNumber

		log.WithFields(log.Fields{
			"message":     inboundMessage,
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": messageBlockNumber,
		}).Info("event is at block")

		finalizedUpdateAfterMessage, err := getFinalizedUpdate(*s, messageBlockNumber)
		if err != nil {
			return err
		}

		finalizedHeaderSlot := uint64(finalizedUpdateAfterMessage.Payload.FinalizedHeader.Slot)

		beaconBlock, blockNumber, err := getBeaconBlockContainingExecutionHeader(*s, messageBlockNumber, finalizedHeaderSlot)
		if err != nil {
			return fmt.Errorf("get beacon block containing header: %w", err)
		}

		beaconBlockSlot, err := strconv.ParseUint(beaconBlock.Data.Message.Slot, 10, 64)
		if err != nil {
			return err
		}

		if blockNumber == messageBlockNumber {
			log.WithFields(log.Fields{
				"slot":        beaconBlock.Data.Message.Slot,
				"blockHash":   beaconBlock.Data.Message.Body.ExecutionPayload.BlockHash,
				"blockNumber": blockNumber,
			}).WithError(err).Info("found execution header containing event")
		}

		checkPoint := cache.Proof{
			FinalizedBlockRoot: finalizedUpdateAfterMessage.FinalizedHeaderBlockRoot,
			BlockRootsTree:     finalizedUpdateAfterMessage.BlockRootsTree,
			Slot:               uint64(finalizedUpdateAfterMessage.Payload.FinalizedHeader.Slot),
		}
		headerUpdateScale, err := s.GetHeaderUpdateBySlotWithCheckpoint(beaconBlockSlot, &checkPoint)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		inboundMessage.Proof.ExecutionProof = headerUpdateScale
		headerUpdate := headerUpdateScale.ToJSON()

		log.WithField("blockNumber", blockNumber).Info("found beacon block by slot")

		messageJSON := inboundMessage.ToJSON()

		finalizedUpdate := finalizedUpdateAfterMessage.Payload.ToJSON()

		finalizedUpdate.RemoveLeadingZeroHashes()
		headerUpdate.RemoveLeadingZeroHashes()
		messageJSON.RemoveLeadingZeroHashes()

		// writing inbound fixture by test case
		testCase, err := cmd.Flags().GetString("test_case")
		if err != nil {
			return err
		}
		if testCase != "register_token" && testCase != "send_token" && testCase != "send_token_to_penpal" && testCase != "send_native_eth" {
			return fmt.Errorf("invalid test case: %s", testCase)
		}

		data := Data{
			FinalizedHeaderUpdate: finalizedUpdate,
			HeaderUpdate:          headerUpdate,
			InboundMessage:        messageJSON,
			TestCase:              testCase,
		}

		rendered, err := mustache.RenderFile(pathToInboundQueueFixtureTestCaseTemplate, data)
		if err != nil {
			return fmt.Errorf("render inbound queue benchmark fixture: %w", err)
		}

		pathToInboundQueueFixtureTestCaseData := fmt.Sprintf(pathToInboundQueueFixtureTestCaseData, testCase)
		err = writeRawDataFile(pathToInboundQueueFixtureTestCaseData, rendered)
		if err != nil {
			return err
		}

		log.Info("done")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon data")
	}

	return nil
}
