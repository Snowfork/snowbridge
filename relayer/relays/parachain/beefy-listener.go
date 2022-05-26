package parachain

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"sort"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/beefyclient"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	ethereumConn        *ethereum.Connection
	beefyClientContract *beefyclient.BeefyClient
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	paraID              uint32
	tasks               chan<- *Task
	eventQueryClient    QueryClient
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

	li.eventQueryClient = NewQueryClient();

	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyClient)
	beefyClientContract, err := beefyclient.NewBeefyClient(address, li.ethereumConn.Client())
	if err != nil {
		return err
	}
	li.beefyClientContract = beefyClientContract

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
			return err
		}

		log.WithFields(log.Fields{
			"blockHash":   beefyBlockHash.Hex(),
			"blockNumber": beefyBlockNumber,
		}).Info("Fetched latest verified polkadot block")

		paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, paraID)
		if err != nil {
			return fmt.Errorf("parachain %v not registered: %w", paraID, err)
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
			return fmt.Errorf("fetch parachain block hash for block %v: %w", paraBlockNumber, err)
		}

		tasks, err := li.discoverCatchupTasks(
			ctx,
			beefyBlockNumber,
			beefyBlockHash,
			paraBlockNumber,
			paraBlockHash,
		)
		if err != nil {
			return fmt.Errorf("discover catchup tasks: %w", err)
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
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}

		return nil
	})

	return nil
}

func (li *BeefyListener) subBeefyJustifications(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)

	sub, err := li.ethereumConn.Client().SubscribeNewHead(ctx, headers)
	if err != nil {
		return fmt.Errorf("creating ethereum header subscription: %w", err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			return fmt.Errorf("header subscription: %w", err)
		case gethheader := <-headers:
			blockNumber := gethheader.Number.Uint64()
			contractEvents, err := li.queryBeefyLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				return fmt.Errorf("query NewMMRRoot event logs in block %v: %w", blockNumber, err)
			}

			if len(contractEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewMMRRoot events in block %d", len(contractEvents), blockNumber))
				// Only process the last emitted event in the block (details in SNO-212)
				err = li.processBeefyLightClientEvent(ctx, contractEvents[len(contractEvents)-1])
				if err != nil {
					return err
				}
			}
		}
	}
}

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyListener) processBeefyLightClientEvent(ctx context.Context, event *beefyclient.BeefyClientNewMMRRoot) error {
	beefyBlockNumber := event.BlockNumber

	log.WithFields(log.Fields{
		"beefyBlockNumber":    beefyBlockNumber,
		"ethereumBlockNumber": event.Raw.BlockNumber,
		"ethereumTxHash":      event.Raw.TxHash.Hex(),
	}).Info("Witnessed a new MMRRoot event")

	beefyBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber))
	if err != nil {
		return fmt.Errorf("fetch block hash for block %v: %w", beefyBlockNumber, err)
	}

	paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, li.paraID)
	if err != nil {
		return fmt.Errorf("parachain %v not registered: %w", li.paraID, err)
	}

	paraBlockNumber := uint64(paraHead.Number)
	paraBlockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return fmt.Errorf("fetch parachain block hash for block %v: %w", paraBlockNumber, err)
	}

	tasks, err := li.discoverCatchupTasks(ctx, beefyBlockNumber, beefyBlockHash, paraBlockNumber, paraBlockHash)
	if err != nil {
		return err
	}

	for _, task := range tasks {

		if task.ProofInput.PolkadotBlockNumber >= beefyBlockNumber {
			log.WithFields(log.Fields{
				"proof.PolkadotBlockNumber": task.ProofInput.PolkadotBlockNumber,
				"beefyBlockNumber": beefyBlockNumber,
			}).Info("Skipping task which is not bounded by latest beefyBlock")
			return nil
		}

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

	return nil
}

// queryBeefyLightClientEvents queries ContractNewMMRRoot events from the BeefyLightClient contract
func (li *BeefyListener) queryBeefyLightClientEvents(
	ctx context.Context, start uint64,
	end *uint64,
) ([]*beefyclient.BeefyClientNewMMRRoot, error) {
	var events []*beefyclient.BeefyClientNewMMRRoot
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyClientContract.FilterNewMMRRoot(&filterOps)
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
	number, err := li.beefyClientContract.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		return 0, types.Hash{}, fmt.Errorf("fetch latest beefy block from light client: %w", err)
	}

	hash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(number)
	if err != nil {
		return 0, types.Hash{}, fmt.Errorf("fetch block hash from relay chain: %w", err)
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
		li.ethereumConn.Client(),
	)
	if err != nil {
		return nil, err
	}

	incentivizedContract, err := incentivized.NewIncentivizedInboundChannel(common.HexToAddress(
		li.config.Contracts.IncentivizedInboundChannel),
		li.ethereumConn.Client(),
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
		return nil, fmt.Errorf("create storage key: %w", err)
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
		return nil, fmt.Errorf("create storage key: %w", err)
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
		ctx,
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
			return fmt.Errorf("decode parachain header: %w", err)
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
		return nil, fmt.Errorf("fetch latest beefy block: %w", err)
	}

	log.WithFields(log.Fields{
		"beefyBlock": latestBeefyBlockNumber,
		"leafIndex": input.PolkadotBlockNumber,
	}).Info("Generating MMR proof")

	mmrProof, err := li.relaychainConn.GenerateProofForBlock(
		input.PolkadotBlockNumber,
		latestBeefyBlockHash,
		li.config.BeefyActivationBlock,
	)
	if err != nil {
		return nil, fmt.Errorf("generate MMR leaf proof: %w", err)
	}

	simplifiedProof, err := merkle.ConvertToSimplifiedMMRProof(
		mmrProof.BlockHash,
		uint64(mmrProof.Proof.LeafIndex),
		mmrProof.Leaf,
		uint64(mmrProof.Proof.LeafCount),
		mmrProof.Proof.Items,
	)
	if err != nil {
		return nil, fmt.Errorf("simplify MMR leaf proof: %w", err)
	}

	mmrRootHashKey, err := types.CreateStorageKey(li.relaychainConn.Metadata(), "Mmr", "RootHash", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key: %w", err)
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
		return nil, fmt.Errorf("create parachain header proof: %w", err)
	}

	if merkleProofData.Root.Hex() != mmrProof.Leaf.ParachainHeads.Hex() {
		return nil, fmt.Errorf("MMR parachain merkle root does not match calculated parachain merkle root (mmr: %s, computed: %s)",
			mmrProof.Leaf.ParachainHeads.Hex(),
			merkleProofData.Root.String(),
		)
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
	ctx context.Context,
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
		}).Debug("Checking header")

		blockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(currentBlockNumber)
		if err != nil {
			return nil, fmt.Errorf("fetch blockhash for block %v: %w", currentBlockNumber, err)
		}

		header, err := li.parachainConnection.API().RPC.Chain.GetHeader(blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch header for %v: %w", blockHash.Hex(), err)
		}

		digestItems, err := ExtractAuxiliaryDigestItems(header.Digest)
		if err != nil {
			return nil, err
		}

		if len(digestItems) == 0 {
			currentBlockNumber--
			continue
		}

		commitments := make(map[ChannelID]Commitment)

		events, err := li.eventQueryClient.QueryEvents(ctx, li.config.Parachain.Endpoint, blockHash)
		if err != nil {
			return nil, fmt.Errorf("query events: %w", err);
		}


		for _, digestItem := range digestItems {
			if !digestItem.IsCommitment {
				continue
			}
			channelID := digestItem.AsCommitment.ChannelID
			if channelID.IsBasic && !scanBasicChannelDone {
				if events.Basic == nil {
					return nil, fmt.Errorf("event basicOutboundChannel.Committed not found in block")
				}

				if events.Basic.Hash != digestItem.AsCommitment.Hash {
					return nil, fmt.Errorf("basic channel commitment hash in digest item does not match the one in the Committed event")
				}

				bundle := events.Basic.Bundle
				bundleNonceBigInt := big.Int(bundle.Nonce)
				bundleNonce := bundleNonceBigInt.Uint64()

				// This case will be hit if basicNonceToFind has not yet
				// been committed yet. Channels emit commitments every N
				// blocks.
				if bundleNonce < basicNonceToFind {
					scanBasicChannelDone = true
					log.Debug("Halting scan. Messages not committed yet on basic channel")
					// Collect these commitments
				} else if bundleNonce > basicNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, bundle)
					// collect this commitment and terminate scan
				} else if bundleNonce == basicNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, bundle)
					scanBasicChannelDone = true
				}
			}
			if channelID.IsIncentivized && !scanIncentivizedChannelDone {
				if events.Incentivized == nil {
					return nil, fmt.Errorf("event basicOutboundChannel.Committed not found in block")
				}

				if events.Incentivized.Hash != digestItem.AsCommitment.Hash {
					return nil, fmt.Errorf("incentivized channel commitment hash in digest item does not match the one in the Committed event")
				}

				bundle := events.Incentivized.Bundle
				bundleNonceBigInt := big.Int(bundle.Nonce)
				bundleNonce := bundleNonceBigInt.Uint64()

				// This case will be hit if basicNonceToFind has not yet
				// been committed yet. Channels emit commitments every N
				// blocks
				if bundleNonce < incentivizedNonceToFind {
					scanIncentivizedChannelDone = true
					continue
					// Collect these commitments
				} else if bundleNonce > incentivizedNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, bundle)
					// collect this commitment and terminate scan
				} else if bundleNonce == incentivizedNonceToFind {
					commitments[channelID] = NewCommitment(digestItem.AsCommitment.Hash, bundle)
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


type OffchainStorageValue struct {
	Nonce      uint64
	Commitment []byte
}


func (li *BeefyListener) fetchOffchainData(digestItem AuxiliaryDigestItem) (*OffchainStorageValue, error) {
	storageKey, err := makeStorageKey(digestItem.AsCommitment.ChannelID, digestItem.AsCommitment.Hash)
	if err != nil {
		return nil, err
	}

	data, err := li.parachainConnection.API().RPC.Offchain.LocalStorageGet(offchain.Persistent, storageKey)
	if err != nil {
		return nil, fmt.Errorf("read commitment from offchain storage: %w", err)
	}
	if data == nil {
		return nil, fmt.Errorf("offchain storage item not found")
	}

	var offchainStorageValue OffchainStorageValue
	err = types.DecodeFromBytes(*data, &offchainStorageValue)
	if err != nil {
		return nil, fmt.Errorf("decode offchain storage value: %w", err)
	}

	return &offchainStorageValue, nil
}
