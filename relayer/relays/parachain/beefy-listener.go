package parachain

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/ofac"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	scheduleConfig      *ScheduleConfig
	ethereumConn        *ethereum.Connection
	beefyClientContract *contracts.BeefyClient
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	ofac                *ofac.OFAC
	paraID              uint32
	tasks               chan<- *Task
	scanner             *Scanner
}

func NewBeefyListener(
	config *SourceConfig,
	scheduleConfig *ScheduleConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	ofac *ofac.OFAC,
	tasks chan<- *Task,
) *BeefyListener {
	return &BeefyListener{
		config:              config,
		scheduleConfig:      scheduleConfig,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		ofac:                ofac,
		tasks:               tasks,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {
	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyClient)
	beefyClientContract, err := contracts.NewBeefyClient(address, li.ethereumConn.Client())
	if err != nil {
		return err
	}
	li.beefyClientContract = beefyClientContract

	// fetch ParaId
	paraIDKey, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "ParachainInfo", "ParachainId", nil, nil)
	if err != nil {
		return err
	}
	var paraID uint32
	ok, err := li.parachainConnection.API().RPC.State.GetStorageLatest(paraIDKey, &paraID)
	if err != nil {
		return fmt.Errorf("fetch parachain id: %w", err)
	}
	if !ok {
		return fmt.Errorf("parachain id missing")
	}
	li.paraID = paraID

	li.scanner = &Scanner{
		config:    li.config,
		ethConn:   li.ethereumConn,
		relayConn: li.relaychainConn,
		paraConn:  li.parachainConnection,
		paraID:    paraID,
		ofac:      li.ofac,
	}

	eg.Go(func() error {
		defer close(li.tasks)

		// Subscribe NewMMRRoot event logs and fetch parachain message commitments
		// since latest beefy block
		beefyBlockNumber, _, err := li.fetchLatestBeefyBlock(ctx)
		if err != nil {
			return fmt.Errorf("fetch latest beefy block: %w", err)
		}

		err = li.doScan(ctx, beefyBlockNumber)
		if err != nil {
			return fmt.Errorf("scan for sync tasks bounded by BEEFY block %v: %w", beefyBlockNumber, err)
		}

		err = li.subscribeNewMMRRoots(ctx)
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

func (li *BeefyListener) subscribeNewMMRRoots(ctx context.Context) error {
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
			contractEvents, err := li.queryBeefyClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				return fmt.Errorf("query NewMMRRoot event logs in block %v: %w", blockNumber, err)
			}

			if len(contractEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewMMRRoot events in block %d", len(contractEvents), blockNumber))
				// Only process the last emitted event in the block
				event := contractEvents[len(contractEvents)-1]
				log.WithFields(log.Fields{
					"beefyBlockNumber":    event.BlockNumber,
					"ethereumBlockNumber": event.Raw.BlockNumber,
					"ethereumTxHash":      event.Raw.TxHash.Hex(),
				}).Info("Witnessed a new MMRRoot event")

				err = li.doScan(ctx, event.BlockNumber)
				if err != nil {
					return fmt.Errorf("scan for sync tasks bounded by BEEFY block %v: %w", event.BlockNumber, err)
				}
			}
		}
	}
}

func (li *BeefyListener) doScan(ctx context.Context, beefyBlockNumber uint64) error {
	tasks, err := li.scanner.Scan(ctx, beefyBlockNumber)
	if err != nil {
		return err
	}
	for _, task := range tasks {
		paraNonce := (*task.MessageProofs)[0].Message.Nonce
		waitingPeriod := (paraNonce + li.scheduleConfig.TotalRelayerCount - li.scheduleConfig.ID) % li.scheduleConfig.TotalRelayerCount
		err = li.waitAndSend(ctx, task, waitingPeriod)
		if err != nil {
			return fmt.Errorf("wait task for nonce %d: %w", paraNonce, err)
		}
	}

	return nil
}

// queryBeefyClientEvents queries ContractNewMMRRoot events from the BeefyClient contract
func (li *BeefyListener) queryBeefyClientEvents(
	ctx context.Context, start uint64,
	end *uint64,
) ([]*contracts.BeefyClientNewMMRRoot, error) {
	var events []*contracts.BeefyClientNewMMRRoot
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
		return 0, types.Hash{}, fmt.Errorf("fetch block hash: %w", err)
	}

	return number, hash, nil
}

// The maximum paras that will be included in the proof.
// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/parachains/src/paras/mod.rs#L1225-L1232
const MaxParaHeads = 1024

// Generates a proof for an MMR leaf, and then generates a merkle proof for our parachain header, which should be verifiable against the
// parachains root in the mmr leaf.
func (li *BeefyListener) generateProof(ctx context.Context, input *ProofInput, header *types.Header) (*ProofOutput, error) {
	latestBeefyBlockNumber, latestBeefyBlockHash, err := li.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return nil, fmt.Errorf("fetch latest beefy block: %w", err)
	}

	log.WithFields(log.Fields{
		"beefyBlock": latestBeefyBlockNumber,
		"leafIndex":  input.RelayBlockNumber,
	}).Info("Generating MMR proof")

	// Generate the MMR proof for the polkadot block.
	mmrProof, err := li.relaychainConn.GenerateProofForBlock(
		input.RelayBlockNumber+1,
		latestBeefyBlockHash,
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

	mmrRootHash, err := li.relaychainConn.GetMMRRootHash(latestBeefyBlockHash)
	if err != nil {
		return nil, fmt.Errorf("retrieve MMR root hash at block %v: %w", latestBeefyBlockHash.Hex(), err)
	}

	var merkleProofData *MerkleProofData
	merkleProofData, input.ParaHeads, err = li.generateAndValidateParasHeadsMerkleProof(input, &mmrProof)
	if err != nil {
		return nil, err
	}

	log.Debug("Created all parachain merkle proof data")

	output := ProofOutput{
		MMRProof:        simplifiedProof,
		MMRRootHash:     mmrRootHash,
		Header:          *header,
		MerkleProofData: *merkleProofData,
	}

	return &output, nil
}

// Generate a merkle proof for the parachain head with input ParaId and verify with merkle root hash of all parachain heads
func (li *BeefyListener) generateAndValidateParasHeadsMerkleProof(input *ProofInput, mmrProof *types.GenerateMMRProofResponse) (*MerkleProofData, []relaychain.ParaHead, error) {
	// Polkadot uses the following code to generate merkle root from parachain headers:
	// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/westend/src/lib.rs#L453-L460
	// Truncate the ParaHeads to the 1024
	// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/parachains/src/paras/mod.rs#L1305-L1311
	paraHeads := input.ParaHeads
	numParas := min(MaxParaHeads, len(paraHeads))
	merkleProofData, err := CreateParachainMerkleProof(paraHeads[:numParas], input.ParaID)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("create parachain header proof: %w", err)
	}

	// Verify merkle root generated is same as value generated in relaychain and if so exit early
	if merkleProofData.Root.Hex() == mmrProof.Leaf.ParachainHeads.Hex() {
		return &merkleProofData, paraHeads, nil
	}

	// Try a filtering out parathreads
	log.WithFields(log.Fields{
		"computedMmr": merkleProofData.Root.Hex(),
		"mmr":         mmrProof.Leaf.ParachainHeads.Hex(),
	}).Warn("MMR parachain merkle root does not match calculated merkle root. Trying to filtering out parathreads.")

	paraHeads, err = li.relaychainConn.FilterParachainHeads(paraHeads, input.RelayBlockHash)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("could not filter out parathreads: %w", err)
	}

	numParas = min(MaxParaHeads, len(paraHeads))
	merkleProofData, err = CreateParachainMerkleProof(paraHeads[:numParas], input.ParaID)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("create parachain header proof: %w", err)
	}
	if merkleProofData.Root.Hex() != mmrProof.Leaf.ParachainHeads.Hex() {
		return nil, paraHeads, fmt.Errorf("MMR parachain merkle root does not match calculated parachain merkle root (mmr: %s, computed: %s)",
			mmrProof.Leaf.ParachainHeads.Hex(),
			merkleProofData.Root.String(),
		)
	}
	return &merkleProofData, paraHeads, nil
}

func (li *BeefyListener) waitAndSend(ctx context.Context, task *Task, waitingPeriod uint64) error {
	paraNonce := (*task.MessageProofs)[0].Message.Nonce
	log.Info(fmt.Sprintf("waiting for nonce %d to be picked up by another relayer", paraNonce))
	var cnt uint64
	var err error
	for {
		ethInboundNonce, err := li.scanner.findLatestNonce(ctx)
		if err != nil {
			return err
		}
		if ethInboundNonce >= paraNonce {
			log.Info(fmt.Sprintf("nonce %d picked up by another relayer, just skip", paraNonce))
			return nil
		}
		if cnt == waitingPeriod {
			break
		}
		time.Sleep(time.Duration(li.scheduleConfig.SleepInterval) * time.Second)
		cnt++
	}
	log.Info(fmt.Sprintf("nonce %d is not picked up by any one, submit anyway", paraNonce))
	task.ProofOutput, err = li.generateProof(ctx, task.ProofInput, task.Header)
	if err != nil {
		return err
	}
	select {
	case <-ctx.Done():
		return ctx.Err()
	case li.tasks <- task:
		log.Info("Beefy Listener emitted new task")
	}
	return nil
}
