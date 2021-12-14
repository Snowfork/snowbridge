package parachain

import (
	"context"
	"fmt"
	"sort"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	ethereumConn        *ethereum.Connection
	beefyLightClient    *beefylightclient.Contract
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	paraID              uint32
	tasks               chan<- *Task
}

func NewBeefyListener(
	config *SourceConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	tasks chan<- *Task,
) *BeefyListener {
	return &BeefyListener{
		config:              config,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		tasks:               tasks,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {

	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.beefyLightClient = beefyLightClientContract

	// Fetch ParaId
	storageKeyForParaID, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "ParachainInfo", "ParachainId", nil, nil)
	if err != nil {
		return err
	}

	var paraID uint32
	ok, err := li.parachainConnection.API().RPC.State.GetStorageLatest(storageKeyForParaID, &paraID)
	if err != nil {
		log.WithError(err).Error("Failed to get para id for snowbridge")
		return err
	}
	if !ok {
		log.Error("Expected parachain but chain does not provide a parachain ID")
		return fmt.Errorf("invalid parachain")
	}

	log.WithField("paraId", paraID).Info("Fetched parachain id")
	li.paraID = paraID

	eg.Go(func() error {
		defer close(li.tasks)

		beefyBlockNumber, beefyBlockHash, err := li.fetchLatestBeefyBlock(ctx)
		if err != nil {
			log.WithError(err).Error("Failed to get latest relay chain block number and hash")
			return err
		}

		log.WithFields(log.Fields{
			"blockHash":   beefyBlockHash.Hex(),
			"blockNumber": beefyBlockNumber,
		}).Info("Fetched latest verified polkadot block")

		paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, paraID)
		if err != nil {
			log.WithError(err).Error("Parachain not registered")
			return err
		}

		log.WithFields(log.Fields{
			"header.ParentHash":     paraHead.ParentHash.Hex(),
			"header.Number":         paraHead.Number,
			"header.StateRoot":      paraHead.StateRoot.Hex(),
			"header.ExtrinsicsRoot": paraHead.ExtrinsicsRoot.Hex(),
			"header.Digest":         paraHead.Digest,
			"parachainId":           paraID,
		}).Info("Fetched finalized header for parachain")

		paraBlockNumber := uint64(paraHead.Number)

		paraBlockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(paraBlockNumber)
		if err != nil {
			log.WithError(err).Error("Failed to get latest finalized para block hash")
			return err
		}

		tasks, err := li.discoverCatchupTasks(
			ctx,
			beefyBlockNumber,
			beefyBlockHash,
			paraBlockNumber,
			paraBlockHash,
		)
		if err != nil {
			log.WithError(err).Error("Failed to discover catchup tasks")
			return err
		}

		for _, task := range tasks {
			log.Info("Beefy listener emitting catchup task")
			task.ProofOutput, err = li.generateProof(ctx, task.ProofInput)
			if err != nil {
				return err
			}
			select {
			case <-ctx.Done():
				return ctx.Err()
			case li.tasks <- task:
			}
		}

		err = li.subBeefyJustifications(ctx)
		return err
	})

	return nil
}

func (li *BeefyListener) subBeefyJustifications(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)

	sub, err := li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)
	if err != nil {
		log.WithError(err).Error("Error creating ethereum header subscription")
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down beefy listener")
			return nil
		case err := <-sub.Err():
			log.WithError(err).Error("Error with ethereum header subscription")
			return err
		case gethheader := <-headers:
			// Query LightClientBridge contract's ContractNewMMRRoot events
			blockNumber := gethheader.Number.Uint64()
			var beefyLightClientEvents []*beefylightclient.ContractNewMMRRoot

			contractEvents, err := li.queryBeefyLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			beefyLightClientEvents = append(beefyLightClientEvents, contractEvents...)

			if len(beefyLightClientEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient ContractNewMMRRoot events on block %d", len(beefyLightClientEvents), blockNumber))
			}

			err = li.processBeefyLightClientEvents(ctx, beefyLightClientEvents)
			if err != nil {
				return err
			}
		}
	}
}

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyListener) processBeefyLightClientEvents(ctx context.Context, events []*beefylightclient.ContractNewMMRRoot) error {
	for _, event := range events {

		beefyBlockNumber := event.BlockNumber

		log.WithFields(log.Fields{
			"beefyBlockNumber":    beefyBlockNumber,
			"ethereumBlockNumber": event.Raw.BlockNumber,
			"ethereumTxHash":      event.Raw.TxHash.Hex(),
		}).Info("Witnessed a new MMRRoot event")

		log.WithField("beefyBlockNumber", beefyBlockNumber).Info("Getting hash for relay chain block")
		beefyBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber))
		if err != nil {
			log.WithError(err).Error("Failed to get block hash")
			return err
		}
		log.WithField("beefyBlockHash", beefyBlockHash.Hex()).Info("Got relay chain blockhash")

		paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, li.paraID)
		if err != nil {
			log.WithError(err).Error("Failed to get finalized para head from relay chain")
			return err
		}

		paraBlockNumber := uint64(paraHead.Number)

		paraBlockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(paraBlockNumber)
		if err != nil {
			log.WithError(err).Error("Failed to get latest finalized para block hash")
			return err
		}

		tasks, err := li.discoverCatchupTasks(ctx, beefyBlockNumber, beefyBlockHash, paraBlockNumber, paraBlockHash)
		if err != nil {
			log.WithError(err).Error("Failed to discover catchup tasks")
			return err
		}

		for _, task := range tasks {
			task.ProofOutput, err = li.generateProof(ctx, task.ProofInput)
			if err != nil {
				return err
			}
			select {
			case <-ctx.Done():
				return ctx.Err()
			case li.tasks <- task:
				log.Info("Beefy Listener emitted new task")
			}
		}
	}
	return nil
}

// queryBeefyLightClientEvents queries ContractNewMMRRoot events from the BeefyLightClient contract
func (li *BeefyListener) queryBeefyLightClientEvents(
	ctx context.Context, start uint64,
	end *uint64,
) ([]*beefylightclient.ContractNewMMRRoot, error) {
	var events []*beefylightclient.ContractNewMMRRoot
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyLightClient.FilterNewMMRRoot(&filterOps)
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

// Fetch the latest verified beefy block number and hash from Ethereum
func (li *BeefyListener) fetchLatestBeefyBlock(ctx context.Context) (uint64, types.Hash, error) {
	number, err := li.beefyLightClient.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		log.WithError(err).Error("Failed to get latest verified beefy block number from ethereum")
		return 0, types.Hash{}, err
	}

	hash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(number)
	if err != nil {
		log.WithError(err).Error("Failed to get latest relay chain block hash from relay chain")
		return 0, types.Hash{}, err
	}

	return number, hash, nil
}

// discoverCatchupTasks finds all the commitments which need to be relayed
func (li *BeefyListener) discoverCatchupTasks(
	ctx context.Context,
	polkadotBlockNumber uint64,
	polkadotBlockHash types.Hash,
	paraBlock uint64,
	paraHash types.Hash,
) ([]*Task, error) {
	basicContract, err := basic.NewBasicInboundChannel(common.HexToAddress(
		li.config.Contracts.BasicInboundChannel),
		li.ethereumConn.GetClient(),
	)
	if err != nil {
		return nil, err
	}

	incentivizedContract, err := incentivized.NewIncentivizedInboundChannel(common.HexToAddress(
		li.config.Contracts.IncentivizedInboundChannel),
		li.ethereumConn.GetClient(),
	)
	if err != nil {
		return nil, err
	}

	options := bind.CallOpts{
		Pending: true,
		Context: ctx,
	}

	ethBasicNonce, err := basicContract.Nonce(&options)
	if err != nil {
		return nil, err
	}
	log.WithFields(log.Fields{
		"nonce": ethBasicNonce,
	}).Info("Checked latest nonce delivered to ethereum basic channel")

	ethIncentivizedNonce, err := incentivizedContract.Nonce(&options)
	if err != nil {
		return nil, err
	}
	log.WithFields(log.Fields{
		"nonce": ethIncentivizedNonce,
	}).Info("Checked latest nonce delivered to ethereum incentivized channel")

	paraBasicNonceKey, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "BasicOutboundChannel", "Nonce", nil, nil)
	if err != nil {
		return nil, err
	}
	var paraBasicNonce types.U64
	ok, err := li.parachainConnection.API().RPC.State.GetStorage(paraBasicNonceKey, &paraBasicNonce, paraHash)
	if err != nil {
		log.Error(err)
		return nil, err
	}
	if !ok {
		paraBasicNonce = 0
	}
	log.WithFields(log.Fields{
		"nonce": uint64(paraBasicNonce),
	}).Info("Checked latest nonce generated by parachain basic channel")

	paraIncentivizedNonceKey, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "IncentivizedOutboundChannel", "Nonce", nil, nil)
	if err != nil {
		log.Error(err)
		return nil, err
	}
	var paraIncentivizedNonce types.U64
	ok, err = li.parachainConnection.API().RPC.State.GetStorage(paraIncentivizedNonceKey, &paraIncentivizedNonce, paraHash)
	if err != nil {
		log.Error(err)
		return nil, err
	}
	if !ok {
		paraIncentivizedNonce = 0
	}
	log.WithFields(log.Fields{
		"nonce": uint64(paraIncentivizedNonce),
	}).Info("Checked latest nonce generated by parachain incentivized channel")

	// Determine which channel commitments we need to scan for.
	var scanBasicChannel, scanIncentivizedChannel bool
	var basicNonceToFind, incentivizedNonceToFind uint64

	if uint64(paraBasicNonce) > ethBasicNonce {
		scanBasicChannel = true
		basicNonceToFind = ethBasicNonce + 1
	}

	if uint64(paraIncentivizedNonce) > ethIncentivizedNonce {
		scanIncentivizedChannel = true
		incentivizedNonceToFind = ethIncentivizedNonce + 1
	}

	if !(scanBasicChannel || scanIncentivizedChannel) {
		return nil, nil
	}

	log.Info("Nonces are mismatched, scanning for commitments that need to be relayed")

	tasks, err := li.scanForCommitments(
		paraBlock,
		scanBasicChannel,
		basicNonceToFind,
		scanIncentivizedChannel,
		incentivizedNonceToFind,
	)
	if err != nil {
		return nil, err
	}

	li.gatherProofInputs(polkadotBlockNumber, polkadotBlockHash, tasks)

	return tasks, nil
}

func (li *BeefyListener) gatherProofInputs(
	polkadotBlockNumber uint64,
	polkadotBlockHash types.Hash,
	tasks []*Task,
) error {
	api := li.relaychainConn.API()

	// build mapping: Parachain block number -> Task
	items := make(map[uint64]*Task)
	for _, task := range tasks {
		items[task.BlockNumber] = task
	}

	for len(items) > 0 && polkadotBlockNumber > 0 {
		paraHeads, err := li.relaychainConn.FetchParaHeads(polkadotBlockHash)
		if err != nil {
			return err
		}

		if _, ok := paraHeads[li.paraID]; !ok {
			return fmt.Errorf("snowbridge is not a registered parachain")
		}

		paraHeadsAsSlice := make([]relaychain.ParaHead, 0, len(paraHeads))
		for _, v := range paraHeads {
			paraHeadsAsSlice = append(paraHeadsAsSlice, v)
		}

		var snowbridgeHeader types.Header
		if err := types.DecodeFromBytes(paraHeads[li.paraID].Data, &snowbridgeHeader); err != nil {
			log.WithError(err).Error("Failed to decode Header")
			return err
		}

		snowbridgeBlockNumber := uint64(snowbridgeHeader.Number)

		if task, ok := items[snowbridgeBlockNumber]; ok {
			task.ProofInput = &ProofInput{
				polkadotBlockNumber,
				paraHeadsAsSlice,
			}
			delete(items, snowbridgeBlockNumber)
		}

		polkadotBlockNumber--
		polkadotBlockHash, err = api.RPC.Chain.GetBlockHash(polkadotBlockNumber)
		if err != nil {
			return err
		}
	}

	if len(items) > 0 {
		return fmt.Errorf("Could not gather all proof inputs")
	}

	return nil
}

func (li *BeefyListener) generateProof(ctx context.Context, input *ProofInput) (*ProofOutput, error) {
	latestBeefyBlockNumber, latestBeefyBlockHash, err := li.fetchLatestBeefyBlock(ctx)
	if err != nil {
		log.WithError(err).Error("Failed to get latest relay chain block number and hash")
		return nil, err
	}

	// The mmr_generateProof(leafIndex, AtBlock) rpc will fail if
	// the following is true. So we'll need to self-terminate and try again.
	if input.PolkadotBlockNumber+1 >= latestBeefyBlockNumber {
		return nil, fmt.Errorf("Not able to create a valid proof this round")
	}

	log.WithField("BeefyBlock", latestBeefyBlockNumber).Info("Beefy BlockNumber")

	// Parachain merkle roots are created 1 block later than the actual parachain headers,
	// so we increment input.PolkadotBlockNumber by 1
	mmrProof, err := li.relaychainConn.GenerateProofForBlock(
		input.PolkadotBlockNumber+1,
		latestBeefyBlockHash,
		li.config.BeefyActivationBlock,
	)
	if err != nil {
		log.WithError(err).Error("Failed to generate mmr proof")
		return nil, err
	}

	simplifiedProof, err := merkle.ConvertToSimplifiedMMRProof(
		mmrProof.BlockHash,
		uint64(mmrProof.Proof.LeafIndex),
		mmrProof.Leaf,
		uint64(mmrProof.Proof.LeafCount),
		mmrProof.Proof.Items,
	)
	if err != nil {
		log.WithError(err).Error("Failed to simplify mmr proof")
		return nil, err
	}

	mmrRootHashKey, err := types.CreateStorageKey(li.relaychainConn.Metadata(), "Mmr", "RootHash", nil, nil)
	if err != nil {
		log.Error(err)
		return nil, err
	}
	var mmrRootHash types.Hash
	ok, err := li.relaychainConn.API().RPC.State.GetStorage(mmrRootHashKey, &mmrRootHash, latestBeefyBlockHash)
	if err != nil {
		log.Error(err)
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("could not get mmr root hash")
	}

	merkleProofData, err := CreateParachainMerkleProof(input.ParaHeads, li.paraID)
	if err != nil {
		log.WithError(err).Error("Failed to create parachain header proof")
		return nil, err
	}

	if merkleProofData.Root.Hex() != mmrProof.Leaf.ParachainHeads.Hex() {
		err = fmt.Errorf("MMR parachain merkle root does not match calculated parachain merkle root - calculated: %s, mmr: %s", merkleProofData.Root.String(), mmrProof.Leaf.ParachainHeads.Hex())
		log.WithError(err).Error("Failed to create parachain merkle root")
		return nil, err
	}

	log.Debug("Created all parachain merkle proof data")

	output := ProofOutput{
		MMRProof:        simplifiedProof,
		MMRRootHash:     mmrRootHash,
		MerkleProofData: merkleProofData,
	}

	return &output, nil
}

// Searches for all lost commitments on each channel from the given parachain block number backwards
// until it finds the given basic and incentivized nonce
func (li *BeefyListener) scanForCommitments(
	lastParaBlockNumber uint64,
	scanBasicChannel bool,
	basicNonceToFind uint64,
	scanIncentivizedChannel bool,
	incentivizedNonceToFind uint64,
) ([]*Task, error) {
	log.WithFields(log.Fields{
		"basicNonce":        basicNonceToFind,
		"incentivizedNonce": incentivizedNonceToFind,
		"latestblockNumber": lastParaBlockNumber,
	}).Debug("Searching backwards from latest block on parachain to find block with nonce")

	currentBlockNumber := lastParaBlockNumber
	scanBasicChannelDone := !scanBasicChannel
	scanIncentivizedChannelDone := !scanIncentivizedChannel

	var tasks []*Task

	for (!scanBasicChannelDone || !scanIncentivizedChannelDone) && currentBlockNumber > 0 {
		log.WithFields(log.Fields{
			"blockNumber": currentBlockNumber,
		}).Debug("Checking header...")

		blockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(currentBlockNumber)
		if err != nil {
			log.WithFields(log.Fields{
				"blockNumber": currentBlockNumber,
			}).WithError(err).Error("Failed to fetch blockhash")
			return nil, err
		}

		header, err := li.parachainConnection.API().RPC.Chain.GetHeader(blockHash)
		if err != nil {
			log.WithError(err).Error("Failed to fetch header")
			return nil, err
		}

		digestItems, err := parachain.ExtractAuxiliaryDigestItems(header.Digest)
		if err != nil {
			return nil, err
		}

		commitments := make(map[parachain.ChannelID]Commitment)

		for _, digestItem := range digestItems {
			if !digestItem.IsCommitment {
				continue
			}
			channelID := digestItem.AsCommitment.ChannelID
			if channelID.IsBasic && !scanBasicChannelDone {
				messages, err := li.parachainConnection.ReadBasicOutboundMessages(digestItem)
				if err != nil {
					return nil, err
				}

				if len(messages) == 0 {
					return nil, fmt.Errorf("Assert len(messages) > 0")
				}

				// This case will be hit if basicNonceToFind has not yet
				// been committed yet. Channels emit commitments every N
				// blocks.
				if messages[0].Nonce < basicNonceToFind {
					scanBasicChannelDone = true
					log.Debug("Halting scan. Messages not committed yet on basic channel")
					// Collect these commitments
				} else if messages[0].Nonce > basicNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, messages)
					// collect this commitment and terminate scan
				} else if messages[0].Nonce == basicNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, messages)
					scanBasicChannelDone = true
				}
			}
			if channelID.IsIncentivized && !scanIncentivizedChannelDone {
				messages, err := li.parachainConnection.ReadIncentivizedOutboundMessages(digestItem)
				if err != nil {
					return nil, err
				}

				if len(messages) == 0 {
					return nil, fmt.Errorf("Assert len(messages) > 0")
				}

				// This case will be hit if basicNonceToFind has not yet
				// been committed yet. Channels emit commitments every N
				// blocks
				if messages[0].Nonce < incentivizedNonceToFind {
					scanIncentivizedChannelDone = true
					continue
					// Collect these commitments
				} else if messages[0].Nonce > incentivizedNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, messages)
					// collect this commitment and terminate scan
				} else if messages[0].Nonce == incentivizedNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, messages)
					scanIncentivizedChannelDone = true
				}
			}
		}

		if len(commitments) > 0 {
			task := Task{
				ParaID:      li.paraID,
				BlockNumber: currentBlockNumber,
				Header:      header,
				Commitments: commitments,
				ProofInput:  nil,
				ProofOutput: nil,
			}
			tasks = append(tasks, &task)
		}

		currentBlockNumber--
	}

	// sort tasks by ascending block number
	sort.SliceStable(tasks, func(i, j int) bool {
		return tasks[i].BlockNumber < tasks[j].BlockNumber
	})

	return tasks, nil
}
