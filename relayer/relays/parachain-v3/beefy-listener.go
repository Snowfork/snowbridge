package parachainV3

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/ofac"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	ethereumConn        *ethereum.Connection
	beefyClientContract *contracts.BeefyClient
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	ofac                *ofac.OFAC
	paraID              uint32
	tasks               chan<- *Task
	scanner             *Scanner
	ignoreNonces        []uint64
	deadNonces          *error_tracking.DeadNonceStore
}

func NewBeefyListener(
	config *SourceConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	ofac *ofac.OFAC,
	tasks chan<- *Task,
	ignoreNonces []uint64,
	deadNonces *error_tracking.DeadNonceStore,
) *BeefyListener {
	return &BeefyListener{
		config:              config,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		ofac:                ofac,
		tasks:               tasks,
		ignoreNonces:        ignoreNonces,
		deadNonces:          deadNonces,
	}
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
//
// beefyRelayBlockOverride, when non-nil, uses that relay-chain block number (and its hash) as the BEEFY / MMR anchor instead of reading
// LatestBeefyBlock from Ethereum. Use this when batching submitFiatShamir with v2_submit in one transaction: on-chain light client is
// still stale until the batch runs, but proofs must match the MMR root that submitFiatShamir will write.
func (li *BeefyListener) generateProof(ctx context.Context, input *ProofInput, header *types.Header, beefyRelayBlockOverride *uint64) (*ProofOutput, error) {
	var latestBeefyBlockNumber uint64
	var latestBeefyBlockHash types.Hash
	var err error
	if beefyRelayBlockOverride != nil {
		latestBeefyBlockNumber = *beefyRelayBlockOverride
		latestBeefyBlockHash, err = li.relaychainConn.API().RPC.Chain.GetBlockHash(latestBeefyBlockNumber)
		if err != nil {
			return nil, fmt.Errorf("fetch relay block hash at beefy anchor %d: %w", latestBeefyBlockNumber, err)
		}
	} else {
		latestBeefyBlockNumber, latestBeefyBlockHash, err = li.fetchLatestBeefyBlock(ctx)
		if err != nil {
			return nil, fmt.Errorf("fetch latest beefy block: %w", err)
		}
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

func (li *BeefyListener) initialize(ctx context.Context) error {
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

	gatewayAddress := common.HexToAddress(li.config.Contracts.Gateway)
	gatewayV2, err := contracts.NewGateway(gatewayAddress, li.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create v2 gateway contract for address '%v': %w", gatewayAddress, err)
	}
	gatewayV1, err := contractsv1.NewGateway(gatewayAddress, li.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create v1 gateway contract for address '%v': %w", gatewayAddress, err)
	}

	li.scanner = &Scanner{
		config:       li.config,
		ethConn:      li.ethereumConn,
		relayConn:    li.relaychainConn,
		paraConn:     li.parachainConnection,
		paraID:       paraID,
		ofac:         li.ofac,
		gatewayV2:    gatewayV2,
		gatewayV1:    gatewayV1,
		ignoreNonces: li.ignoreNonces,
		deadNonces:   li.deadNonces,
	}
	return nil
}

// rebindEthereumContracts recreates BeefyClient / Gateway bindings after
// ethereumConn.Reconnect so ChannelNoncesOf and light-client reads use the
// new RPC. LiveClient already forwards; this matches E→P reconnectAndRebind.
func (li *BeefyListener) rebindEthereumContracts() error {
	if li.ethereumConn == nil || li.ethereumConn.Client() == nil {
		return fmt.Errorf("ethereum connection not ready")
	}

	address := common.HexToAddress(li.config.Contracts.BeefyClient)
	beefyClientContract, err := contracts.NewBeefyClient(address, li.ethereumConn.Client())
	if err != nil {
		return err
	}
	li.beefyClientContract = beefyClientContract

	gatewayAddress := common.HexToAddress(li.config.Contracts.Gateway)
	gatewayV2, err := contracts.NewGateway(gatewayAddress, li.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("rebind v2 gateway contract for address '%v': %w", gatewayAddress, err)
	}
	gatewayV1, err := contractsv1.NewGateway(gatewayAddress, li.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("rebind v1 gateway contract for address '%v': %w", gatewayAddress, err)
	}

	if li.scanner != nil {
		li.scanner.gatewayV2 = gatewayV2
		li.scanner.gatewayV1 = gatewayV1
	}
	return nil
}
