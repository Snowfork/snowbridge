package parachainV3

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"slices"
	"strings"
	"sync"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/ofac"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
)

type Scanner struct {
	config       *SourceConfig
	ethConn      *ethereum.Connection
	relayConn    *relaychain.Connection
	paraConn     *parachain.Connection
	paraID       uint32
	ofac         *ofac.OFAC
	tasks        chan<- *Task
	gatewayV2    *contracts.Gateway
	gatewayV1    *contractsv1.Gateway
	ignoreNonces []uint64
	deadNonces   *error_tracking.DeadNonceStore
	// dispatchedCache records nonces for which v2_isDispatched has returned true.
	// Those bits are immutable on-chain, so we never re-query them (cuts repeated
	// eth_calls for stale PendingOrders that remain in BH storage).
	dispatchedMu    sync.Mutex
	dispatchedCache map[uint64]struct{}
}

const v1BackwardScanDepthLimit uint64 = 43200

func (s *Scanner) isIgnoredNonce(nonce uint64) bool {
	if s == nil {
		return false
	}
	return slices.Contains(s.ignoreNonces, nonce)
}

// isDeadNonce reports whether the nonce is recorded in the dead-nonce store.
// The store is populated from the file at DeadNoncesPath, which operators can
// edit to permanently skip un-relayable nonces across restarts.
func (s *Scanner) isDeadNonce(nonce uint64) bool {
	if s == nil || s.deadNonces == nil {
		return false
	}
	return s.deadNonces.IsDead(nonce)
}

// markDeadNonce records a permanently un-relayable nonce in the dead-nonce
// store so it is skipped on subsequent scans (and across restarts when a
// DeadNoncesPath is configured).
func (s *Scanner) markDeadNonce(nonce uint64, reason string) {
	if s == nil || s.deadNonces == nil {
		return
	}
	if err := s.deadNonces.MarkDead(nonce, reason); err != nil {
		log.WithError(err).WithField("nonce", nonce).
			Error("failed to persist dead-nonce record")
	}
}

// Scans for all parachain message commitments that need to be relayed and can be
// proven using the MMR root at the specified beefyBlockNumber of the relay chain.
// The algorithm fetch PendingOrders storage in OutboundQueue of BH and
// just relay each order which has not been processed on Ethereum yet.
func (s *Scanner) Scan(ctx context.Context, beefyBlockNumber uint64) ([]*Task, error) {
	// fetch last parachain header that was finalized *before* the BEEFY block
	beefyBlockMinusOneHash, err := s.relayConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber - 1))
	if err != nil {
		return nil, fmt.Errorf("fetch block hash for block %v: %w", beefyBlockNumber, err)
	}
	var paraHead types.Header
	ok, err := s.relayConn.FetchParachainHead(beefyBlockMinusOneHash, s.paraID, &paraHead)
	if err != nil {
		return nil, fmt.Errorf("fetch head for parachain %v at block %v: %w", s.paraID, beefyBlockMinusOneHash.Hex(), err)
	}
	if !ok {
		return nil, fmt.Errorf("parachain %v is not registered", s.paraID)
	}

	paraBlockNumber := uint64(paraHead.Number)
	paraBlockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch parachain block hash for block %v: %w", paraBlockNumber, err)
	}

	tasks, err := s.findTasks(ctx, paraBlockNumber, paraBlockHash)
	if err != nil {
		return nil, err
	}

	return tasks, nil
}

// ScanV2 scans only for V2 pending messages against the given beefy block.
// Used by the V2-only fast path; OFAC checks are skipped here for latency (legacy
// Scan → findTasks still enforces OFAC).
func (s *Scanner) ScanV2(ctx context.Context, beefyBlockNumber uint64) ([]*Task, error) {
	// fetch last parachain header that was finalized *before* the BEEFY block
	beefyBlockMinusOneHash, err := s.relayConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber - 1))
	if err != nil {
		return nil, fmt.Errorf("fetch block hash for block %v: %w", beefyBlockNumber, err)
	}
	var paraHead types.Header
	ok, err := s.relayConn.FetchParachainHead(beefyBlockMinusOneHash, s.paraID, &paraHead)
	if err != nil {
		return nil, fmt.Errorf("fetch head for parachain %v at block %v: %w", s.paraID, beefyBlockMinusOneHash.Hex(), err)
	}
	if !ok {
		return nil, fmt.Errorf("parachain %v is not registered", s.paraID)
	}

	paraBlockNumber := uint64(paraHead.Number)
	paraBlockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch parachain block hash for block %v: %w", paraBlockNumber, err)
	}

	return s.findV2Tasks(ctx, paraBlockNumber, paraBlockHash)
}

// ScanV1 scans only for V1 pending messages against the given beefy block.
// Used by the fast path (instant + missed-WS); OFAC is not applied to V1 today
// (legacy findV1TasksBackward also skips OFAC). Header resolution matches ScanV2.
func (s *Scanner) ScanV1(ctx context.Context, beefyBlockNumber uint64) ([]*Task, error) {
	beefyBlockMinusOneHash, err := s.relayConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber - 1))
	if err != nil {
		return nil, fmt.Errorf("fetch block hash for block %v: %w", beefyBlockNumber, err)
	}
	var paraHead types.Header
	ok, err := s.relayConn.FetchParachainHead(beefyBlockMinusOneHash, s.paraID, &paraHead)
	if err != nil {
		return nil, fmt.Errorf("fetch head for parachain %v at block %v: %w", s.paraID, beefyBlockMinusOneHash.Hex(), err)
	}
	if !ok {
		return nil, fmt.Errorf("parachain %v is not registered", s.paraID)
	}

	paraBlockNumber := uint64(paraHead.Number)
	paraBlockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch parachain block hash for block %v: %w", paraBlockNumber, err)
	}

	v1Tasks, err := s.findV1TasksBackward(ctx, paraBlockNumber, paraBlockHash)
	if err != nil {
		return nil, fmt.Errorf("find v1 tasks: %w", err)
	}
	if err := s.gatherProofInputs(v1Tasks); err != nil {
		return nil, fmt.Errorf("gather proof input: %w", err)
	}
	return v1Tasks, nil
}

// findTasks finds all the message commitments which need to be relayed
func (s *Scanner) findTasks(
	ctx context.Context,
	paraBlockNumber uint64,
	paraHash types.Hash,
) ([]*Task, error) {
	// Fetch PendingOrders storage in parachain outbound queue
	storageKey := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueueV2", "PendingOrders"))
	keys, err := s.paraConn.API().RPC.State.GetKeys(storageKey, paraHash)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v' and hash '%v': %w", storageKey, paraHash, err)
	}
	var pendingOrders []parachain.PendingOrder
	for _, key := range keys {
		var pendingOrder parachain.PendingOrder
		value, err := s.paraConn.API().RPC.State.GetStorageRaw(key, paraHash)
		if err != nil {
			return nil, fmt.Errorf("fetch value of pendingOrder with key '%v' and hash '%v': %w", key, paraHash, err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*value))
		err = decoder.Decode(&pendingOrder)
		if err != nil {
			return nil, fmt.Errorf("decode order error: %w", err)
		}
		pendingOrders = append(pendingOrders, pendingOrder)
	}

	v2Tasks, err := s.filterTasks(ctx, pendingOrders, false /* skipOFAC */)
	if err != nil {
		return nil, err
	}
	v1Tasks, err := s.findV1TasksBackward(ctx, paraBlockNumber, paraHash)
	if err != nil {
		return nil, fmt.Errorf("find v1 tasks: %w", err)
	}

	tasks := make([]*Task, 0, len(v2Tasks)+len(v1Tasks))
	tasks = append(tasks, v2Tasks...)
	tasks = append(tasks, v1Tasks...)

	err = s.gatherProofInputs(tasks)
	if err != nil {
		return nil, fmt.Errorf("gather proof input: %w", err)
	}

	return tasks, nil
}

func (s *Scanner) findV2Tasks(
	ctx context.Context,
	paraBlockNumber uint64,
	paraHash types.Hash,
) ([]*Task, error) {
	// Fetch PendingOrders storage in parachain outbound queue
	storageKey := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueueV2", "PendingOrders"))
	keys, err := s.paraConn.API().RPC.State.GetKeys(storageKey, paraHash)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v' and hash '%v': %w", storageKey, paraHash, err)
	}
	var pendingOrders []parachain.PendingOrder
	for _, key := range keys {
		var pendingOrder parachain.PendingOrder
		value, err := s.paraConn.API().RPC.State.GetStorageRaw(key, paraHash)
		if err != nil {
			return nil, fmt.Errorf("fetch value of pendingOrder with key '%v' and hash '%v': %w", key, paraHash, err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*value))
		err = decoder.Decode(&pendingOrder)
		if err != nil {
			return nil, fmt.Errorf("decode order error: %w", err)
		}
		pendingOrders = append(pendingOrders, pendingOrder)
	}

	v2Tasks, err := s.filterTasks(ctx, pendingOrders, true /* skipOFAC */)
	if err != nil {
		return nil, err
	}

	err = s.gatherProofInputs(v2Tasks)
	if err != nil {
		return nil, fmt.Errorf("gather proof input: %w", err)
	}

	return v2Tasks, nil
}

func (s *Scanner) findV1TasksBackward(ctx context.Context, lastParaBlockNumber uint64, latestParaHash types.Hash) ([]*Task, error) {
	noncePrefix := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueue", "Nonce"))
	nonceKeys, err := s.paraConn.API().RPC.State.GetKeys(noncePrefix, latestParaHash)
	if err != nil {
		return nil, fmt.Errorf("fetch v1 channel nonce keys: %w", err)
	}
	if len(nonceKeys) == 0 {
		return nil, nil
	}

	gatewayAddress := common.HexToAddress(s.config.Contracts.Gateway)
	gatewayContract, err := contractsv1.NewGateway(gatewayAddress, s.ethConn.Client())
	if err != nil {
		return nil, fmt.Errorf("create v1 gateway contract for address '%v': %w", gatewayAddress, err)
	}

	allTasks := make([]*Task, 0)
	for _, nonceKey := range nonceKeys {
		if len(nonceKey) < 32 {
			continue
		}

		var channelID types.H256
		copy(channelID[:], nonceKey[len(nonceKey)-32:])

		var paraNonce types.U64
		ok, err := s.paraConn.API().RPC.State.GetStorage(nonceKey, &paraNonce, latestParaHash)
		if err != nil {
			return nil, fmt.Errorf("fetch v1 nonce from parachain storage key '%v': %w", nonceKey, err)
		}
		if !ok {
			continue
		}

		ethInboundNonce, _, err := gatewayContract.ChannelNoncesOf(&bind.CallOpts{Pending: true, Context: ctx}, [32]byte(channelID))
		if err != nil {
			return nil, fmt.Errorf("fetch v1 nonce from gateway contract for channelID '%v': %w", channelID.Hex(), err)
		}

		if uint64(paraNonce) <= ethInboundNonce {
			continue
		}

		channelTasks, err := s.scanV1TasksForChannel(ctx, lastParaBlockNumber, channelID, ethInboundNonce+1)
		if err != nil {
			return nil, fmt.Errorf("scan v1 tasks for channel '%v': %w", channelID.Hex(), err)
		}
		allTasks = append(allTasks, channelTasks...)
	}

	return allTasks, nil
}

func (s *Scanner) scanV1TasksForChannel(ctx context.Context, lastParaBlockNumber uint64, channelID types.H256, startingNonce uint64) ([]*Task, error) {
	messagesKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "EthereumOutboundQueue", "Messages", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create v1 messages storage key: %w", err)
	}
	if startingNonce == 0 {
		return nil, fmt.Errorf("invalid starting nonce for channel '%v': %d", channelID.Hex(), startingNonce)
	}

	var tasks []*Task
	scanDone := false
	var scannedBlocks uint64

	for currentBlockNumber := lastParaBlockNumber; currentBlockNumber > 0 && !scanDone && scannedBlocks < v1BackwardScanDepthLimit; currentBlockNumber-- {
		scannedBlocks++
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		blockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(currentBlockNumber)
		if err != nil {
			return nil, fmt.Errorf("fetch block hash for block %v: %w", currentBlockNumber, err)
		}

		header, err := s.paraConn.API().RPC.Chain.GetHeader(blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch header for block hash %v: %w", blockHash.Hex(), err)
		}

		commitmentHash, err := ExtractCommitmentFromDigestV1(header.Digest)
		if err != nil {
			return nil, err
		}
		if commitmentHash == nil {
			continue
		}

		raw, err := s.paraConn.API().RPC.State.GetStorageRaw(messagesKey, blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch v1 committed messages for block %v: %w", blockHash.Hex(), err)
		}
		if raw == nil {
			continue
		}

		decoder := scale.NewDecoder(bytes.NewReader(*raw))
		n, err := decoder.DecodeUintCompact()
		if err != nil {
			return nil, fmt.Errorf("decode v1 message length error: %w", err)
		}

		messages := make([]V1OutboundQueueMessage, 0, n.Uint64())
		for i := uint64(0); i < n.Uint64(); i++ {
			m := V1OutboundQueueMessage{}
			if err := decoder.Decode(&m); err != nil {
				return nil, fmt.Errorf("decode v1 message error: %w", err)
			}
			messages = append(messages, m)
		}

		result, err := scanForOutboundQueueProofsV1(s.paraConn.API(), blockHash, *commitmentHash, startingNonce, channelID, messages)
		if err != nil {
			return nil, err
		}
		scanDone = result.scanDone

		if len(result.proofs) > 0 {
			task := &Task{
				Header:          header,
				ProofInput:      nil,
				ProofOutput:     nil,
				MessageProofs:   nil,
				V1MessageProofs: &result.proofs,
			}
			tasks = append(tasks, task)
		}
	}

	for i, j := 0, len(tasks)-1; i < j; i, j = i+1, j-1 {
		tasks[i], tasks[j] = tasks[j], tasks[i]
	}
	if !scanDone {
		return nil, fmt.Errorf(
			"v1 scan reached depth limit %d without finding starting nonce %d for channel '%v'",
			v1BackwardScanDepthLimit,
			startingNonce,
			channelID.Hex(),
		)
	}

	return tasks, nil
}

func scanForOutboundQueueProofsV1(
	api *gsrpc.SubstrateAPI,
	blockHash types.Hash,
	commitmentHash types.H256,
	startingNonce uint64,
	channelID types.H256,
	messages []V1OutboundQueueMessage,
) (*struct {
	proofs   []V1MessageProof
	scanDone bool
}, error) {
	var scanDone bool
	proofs := []V1MessageProof{}

	for i := len(messages) - 1; i >= 0; i-- {
		message := messages[i]
		if message.ChannelID != channelID {
			continue
		}

		messageNonce := message.Nonce
		if messageNonce < startingNonce {
			scanDone = true
			break
		}

		messageProof, err := fetchV1MessageProof(api, blockHash, uint64(i), message)
		if err != nil {
			return nil, err
		}
		if messageProof.Proof.Root != commitmentHash {
			return nil, fmt.Errorf(
				"v1 outbound queue proof root '%v' doesn't match digest commitment '%v'",
				messageProof.Proof.Root,
				commitmentHash,
			)
		}

		proofs = append(proofs, messageProof)
		if messageNonce == startingNonce {
			scanDone = true
		}
	}

	for i, j := 0, len(proofs)-1; i < j; i, j = i+1, j-1 {
		proofs[i], proofs[j] = proofs[j], proofs[i]
	}

	return &struct {
		proofs   []V1MessageProof
		scanDone bool
	}{
		proofs:   proofs,
		scanDone: scanDone,
	}, nil
}

// Filter profitable and undelivered orders, convert to tasks
// Todo: check order is profitable or not with some price oracle
// or some fee estimation api
func (s *Scanner) filterTasks(
	ctx context.Context,
	pendingOrders []parachain.PendingOrder,
	skipOFAC bool,
) ([]*Task, error) {

	var tasks []*Task

	for _, order := range pendingOrders {

		nonce := uint64(order.Nonce)
		if s.isIgnoredNonce(nonce) {
			log.WithField("nonce", nonce).Debug("skipping configured ignored nonce")
			continue
		}
		if s.isDeadNonce(nonce) {
			log.WithField("nonce", nonce).Debug("skipping dead-lettered nonce")
			continue
		}

		isRelayed, err := s.isNonceRelayed(ctx, nonce)
		if err != nil {
			return nil, fmt.Errorf("check nonce relayed: %w", err)
		}
		if isRelayed {
			log.WithFields(log.Fields{
				"nonce": nonce,
			}).Debug("already relayed, just skip")
			continue
		}

		messagesKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "EthereumOutboundQueueV2", "Messages", nil, nil)
		if err != nil {
			return nil, fmt.Errorf("create storage key: %w", err)
		}

		currentBlockNumber := uint64(order.BlockNumber)

		log.WithFields(log.Fields{
			"blockNumber": currentBlockNumber,
		}).Debug("Checking header")

		blockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(currentBlockNumber)
		if err != nil {
			return nil, fmt.Errorf("fetch block hash for block %v: %w", currentBlockNumber, err)
		}

		header, err := s.paraConn.API().RPC.Chain.GetHeader(blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch header for block hash %v: %w", blockHash.Hex(), err)
		}

		commitmentHash, err := ExtractCommitmentFromDigest(header.Digest)
		if err != nil {
			return nil, err
		}
		if commitmentHash == nil {
			continue
		}

		var messagesWithFee []OutboundQueueMessageWithFee
		raw, err := s.paraConn.API().RPC.State.GetStorageRaw(messagesKey, blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch committed messages for block %v: %w", blockHash.Hex(), err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*raw))
		n, err := decoder.DecodeUintCompact()
		if err != nil {
			return nil, fmt.Errorf("decode message length error: %w", err)
		}
		for i := uint64(0); i < n.Uint64(); i++ {
			m := OutboundQueueMessage{}
			err = decoder.Decode(&m)
			if err != nil {
				return nil, fmt.Errorf("decode message error: %w", err)
			}
			if !skipOFAC {
				isBanned, err := s.IsBanned(m)
				if err != nil {
					log.WithError(err).Fatal("error checking banned address found")
					return nil, fmt.Errorf("banned check: %w", err)
				}
				if isBanned {
					log.Fatal("banned address found")
					return nil, errors.New("banned address found")
				}
			}
			if isBlockedV2Origin(m.Origin) {
				log.WithField("nonce", uint64(m.Nonce)).
					Fatal(fmt.Sprintf("blocked Asset Hub root origin %s found", blockedV2OriginHex))
			}
			var messageWithFee OutboundQueueMessageWithFee
			messageWithFee.OriginalMessage = m
			messageWithFee.Fee = order.Fee
			messagesWithFee = append(messagesWithFee, messageWithFee)
		}

		var messageLeaves []types.H256
		messageLeavesKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "EthereumOutboundQueueV2", "MessageLeaves", nil, nil)
		if err != nil {
			return nil, fmt.Errorf("create storage key: %w", err)
		}
		_, err = s.paraConn.API().RPC.State.GetStorage(messageLeavesKey, &messageLeaves, blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch message leaves for block %v: %w", blockHash.Hex(), err)
		}

		result, err := buildOutboundQueueProofs(
			*commitmentHash,
			messagesWithFee,
			messageLeaves,
		)
		if err != nil {
			return nil, err
		}

		if len(result.proofs) > 0 {
			task := Task{
				Header:        header,
				MessageProofs: &result.proofs,
				ProofInput:    nil,
				ProofOutput:   nil,
			}
			tasks = append(tasks, &task)
		}
	}

	return tasks, nil
}

// For each task, gatherProofInputs will search to find the relay chain block
// in which that header was included as well as the parachain heads for that block.
func (s *Scanner) gatherProofInputs(
	tasks []*Task,
) error {
	for _, task := range tasks {

		log.WithFields(log.Fields{
			"ParaBlockNumber": task.Header.Number,
		}).Debug("Gathering proof inputs for parachain header")

		relayBlockNumber, err := s.findInclusionBlockNumber(uint64(task.Header.Number))
		if err != nil {
			return fmt.Errorf("find inclusion block number for parachain block %v: %w", task.Header.Number, err)
		}

		relayBlockHash, err := s.relayConn.API().RPC.Chain.GetBlockHash(relayBlockNumber)
		if err != nil {
			return fmt.Errorf("fetch relaychain block hash: %w", err)
		}

		parachainHeads, err := s.relayConn.FetchParasHeads(relayBlockHash)
		if err != nil {
			return fmt.Errorf("fetch parachain heads: %w", err)
		}

		task.ProofInput = &ProofInput{
			ParaID:           s.paraID,
			RelayBlockNumber: relayBlockNumber,
			RelayBlockHash:   relayBlockHash,
			ParaHeads:        parachainHeads,
		}
	}

	return nil
}

// Find the relaychain block in which a parachain header was included (finalized). This usually happens
// 2-3 blocks after the relaychain block in which the parachain header was backed.
func (s *Scanner) findInclusionBlockNumber(
	paraBlockNumber uint64,
) (uint64, error) {
	validationDataKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "ParachainSystem", "ValidationData", nil, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key: %w", err)
	}

	paraBlockHash, err := s.paraConn.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return 0, fmt.Errorf("fetch parachain block hash: %w", err)
	}

	var validationData parachain.PersistedValidationData
	ok, err := s.paraConn.API().RPC.State.GetStorage(validationDataKey, &validationData, paraBlockHash)
	if err != nil {
		return 0, fmt.Errorf("fetch PersistedValidationData for block %v: %w", paraBlockHash.Hex(), err)
	}
	if !ok {
		return 0, fmt.Errorf("PersistedValidationData not found for block %v", paraBlockHash.Hex())
	}

	startBlock := validationData.RelayParentNumber + 1
	for i := validationData.RelayParentNumber + 1; i < startBlock+relaychain.FinalizationTimeout; i++ {
		relayBlockHash, err := s.relayConn.API().RPC.Chain.GetBlockHash(uint64(i))
		if err != nil {
			return 0, fmt.Errorf("fetch relaychain block hash: %w", err)
		}

		var paraHead types.Header
		ok, err := s.relayConn.FetchParachainHead(relayBlockHash, s.paraID, &paraHead)
		if err != nil {
			return 0, fmt.Errorf("fetch head for parachain %v at block %v: %w", s.paraID, relayBlockHash.Hex(), err)
		}
		if !ok {
			return 0, fmt.Errorf("parachain %v is not registered", s.paraID)
		}

		if paraBlockNumber == uint64(paraHead.Number) {
			return uint64(i), nil
		}
	}

	return 0, fmt.Errorf("scan terminated")
}

func (s *Scanner) isCachedDispatched(nonce uint64) bool {
	if s == nil {
		return false
	}
	s.dispatchedMu.Lock()
	defer s.dispatchedMu.Unlock()
	_, ok := s.dispatchedCache[nonce]
	return ok
}

func (s *Scanner) markDispatched(nonce uint64) {
	if s == nil {
		return
	}
	s.dispatchedMu.Lock()
	defer s.dispatchedMu.Unlock()
	if s.dispatchedCache == nil {
		s.dispatchedCache = make(map[uint64]struct{})
	}
	s.dispatchedCache[nonce] = struct{}{}
}

func (s *Scanner) isNonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	if s.isCachedDispatched(nonce) {
		log.WithField("nonce", nonce).Debug("already relayed (cached), just skip")
		return true, nil
	}
	if s.gatewayV2 == nil {
		return false, fmt.Errorf("v2 gateway contract is not initialized")
	}

	options := bind.CallOpts{
		Pending: true,
		Context: ctx,
	}
	isRelayed, err := s.gatewayV2.V2IsDispatched(&options, nonce)
	if err != nil {
		return false, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	if isRelayed {
		s.markDispatched(nonce)
	}
	return isRelayed, nil
}

func (s *Scanner) isV1NonceRelayed(ctx context.Context, channelID [32]byte, nonce uint64) (bool, error) {
	if s.gatewayV1 == nil {
		return false, fmt.Errorf("v1 gateway contract is not initialized")
	}

	options := bind.CallOpts{Pending: true, Context: ctx}
	ethInboundNonce, _, err := s.gatewayV1.ChannelNoncesOf(&options, channelID)
	if err != nil {
		return false, fmt.Errorf("fetch v1 nonce from gateway contract for channelID '%v': %w", types.H256(channelID).Hex(), err)
	}
	return ethInboundNonce >= nonce, nil
}

func fetchV1MessageProof(
	api *gsrpc.SubstrateAPI,
	blockHash types.Hash,
	messageIndex uint64,
	message V1OutboundQueueMessage,
) (V1MessageProof, error) {
	var proofHex string

	params, err := types.EncodeToHexString(messageIndex)
	if err != nil {
		return V1MessageProof{}, fmt.Errorf("encode params: %w", err)
	}

	err = api.Client.Call(&proofHex, "state_call", "OutboundQueueApi_prove_message", params, blockHash.Hex())
	if err != nil {
		return V1MessageProof{}, fmt.Errorf("call RPC OutboundQueueApi_prove_message(%v, %v): %w", messageIndex, blockHash, err)
	}

	var optionRawMerkleProof OptionRawMerkleProof
	err = types.DecodeFromHexString(proofHex, &optionRawMerkleProof)
	if err != nil {
		return V1MessageProof{}, fmt.Errorf("decode merkle proof: %w", err)
	}
	if !optionRawMerkleProof.HasValue {
		return V1MessageProof{}, fmt.Errorf("retrieve v1 proof failed")
	}

	proof, err := NewMerkleProof(optionRawMerkleProof.Value)
	if err != nil {
		return V1MessageProof{}, fmt.Errorf("decode merkle proof: %w", err)
	}

	return V1MessageProof{Message: message, Proof: proof}, nil
}

func (s *Scanner) IsBanned(m OutboundQueueMessage) (bool, error) {
	destinations, err := GetDestinations(m)
	if err != nil {
		return true, err
	}

	isBanned, err := s.ofac.IsBanned("", destinations)
	if err != nil {
		return true, err
	}

	return isBanned, nil
}

func GetDestinations(message OutboundQueueMessage) ([]string, error) {
	var destinations []string
	log.WithFields(log.Fields{
		"commands": message.Commands,
	}).Debug("Checking message for OFAC")

	address := ""

	bytes32Ty, _ := abi.NewType("bytes32", "", nil)
	addressTy, _ := abi.NewType("address", "", nil)
	uint256Ty, _ := abi.NewType("uint256", "", nil)
	for _, command := range message.Commands {
		switch command.Kind {
		case 2:
			log.Debug("Unlock native token")

			uintTy, _ := abi.NewType("uint256", "", nil)
			transferTokenArgument := abi.Arguments{
				{Type: addressTy},
				{Type: addressTy},
				{Type: uintTy},
			}
			decodedTransferToken, err := transferTokenArgument.Unpack(command.Params)
			if err != nil {
				return destinations, err
			}
			if len(decodedTransferToken) < 3 {
				return destinations, errors.New("decode transfer token command")
			}

			addressValue := decodedTransferToken[1].(common.Address)
			address = addressValue.String()
		case 4:
			log.Debug("Found MintForeignToken message")

			arguments := abi.Arguments{
				{Type: bytes32Ty},
				{Type: addressTy},
				{Type: uint256Ty},
			}

			decodedMessage, err := arguments.Unpack(command.Params)
			if err != nil {
				return destinations, fmt.Errorf("unpack tuple: %w", err)
			}
			if len(decodedMessage) < 3 {
				return destinations, fmt.Errorf("decoded message not found")
			}

			addressValue := decodedMessage[1].(common.Address)
			address = addressValue.String()
		}

		destination := strings.ToLower(address)

		log.WithField("destination", destination).Debug("extracted destination from message")

		destinations = append(destinations, destination)
	}

	return destinations, nil
}

func buildOutboundQueueProofs(
	commitmentHash types.H256,
	messages []OutboundQueueMessageWithFee,
	messageLeaves []types.H256,
) (*struct {
	proofs []MessageProof
}, error) {

	Keccak256Contents := []merkle.Content{}
	for _, leaf := range messageLeaves {
		var content merkle.Keccak256Content
		copy(content.X[:], leaf[:])
		Keccak256Contents = append(Keccak256Contents, content)
	}

	tree, err := merkle.NewTree2(Keccak256Contents)
	if err != nil {
		return nil, err
	}
	root := tree.Root.Hash
	if !bytes.Equal(root, commitmentHash[:]) {
		return nil, fmt.Errorf(
			"outbound queue computed merkle root '%v' doesn't match digest item's commitment hash '%v'",
			root,
			commitmentHash,
		)
	}

	proofs := []MessageProof{}

	for i := len(messages) - 1; i >= 0; i-- {
		message := messages[i]
		messageLeaf := messageLeaves[i]

		var content merkle.Keccak256Content
		copy(content.X[:], messageLeaf[:])

		messagePath, _, err := tree.MerklePath(content)
		if err != nil {
			return nil, fmt.Errorf("get merkle path: %w", err)
		}

		byteArrayProof := make([][32]byte, len(messagePath))
		for i := 0; i < len(messagePath); i++ {
			byteArrayProof[i] = ([32]byte)(messagePath[i])
		}

		proof := MerkleProof{
			Root:        commitmentHash,
			InnerHashes: byteArrayProof,
		}

		messageProof := MessageProof{Message: message, Proof: proof}

		// Collect these commitments
		proofs = append(proofs, messageProof)
	}

	return &struct {
		proofs []MessageProof
	}{
		proofs: proofs,
	}, nil
}
