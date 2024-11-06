package parachain

import (
	"bytes"
	"context"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

type Scanner struct {
	config    *SourceConfig
	ethConn   *ethereum.Connection
	relayConn *relaychain.Connection
	paraConn  *parachain.Connection
	paraID    uint32
	tasks     chan<- *Task
}

// Scans for all parachain message commitments that need to be relayed and can be
// proven using the MMR root at the specified beefyBlockNumber of the relay chain.
// The algorithm fetch PendingOrders storage in OutboundQueue of BH and
// just relay each order which has not been processed on Ethereum yet.
func (s *Scanner) Scan(ctx context.Context, beefyBlockNumber uint64) ([]*TaskV2, error) {
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

	tasks, err := s.findTasks(ctx, paraBlockHash)
	if err != nil {
		return nil, err
	}

	return tasks, nil
}

// findTasks finds all the message commitments which need to be relayed
func (s *Scanner) findTasks(
	ctx context.Context,
	paraHash types.Hash,
) ([]*TaskV2, error) {
	// Fetch PendingOrders storage in parachain outbound queue
	storageKey := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueueV2", "PendingOrders"))
	keys, err := s.paraConn.API().RPC.State.GetKeys(storageKey, paraHash)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v' and hash '%v': %w", storageKey, paraHash, err)
	}
	var pendingOrders []PendingOrder
	for _, key := range keys {
		var pendingOrder PendingOrder
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

	tasks, err := s.findTasksImpl(
		ctx,
		pendingOrders,
	)
	if err != nil {
		return nil, err
	}

	s.gatherProofInputs(tasks)

	return tasks, nil
}

// Searches from for all outstanding commitments
func (s *Scanner) findTasksImpl(
	ctx context.Context,
	pendingNonces []PendingOrder,
) ([]*TaskV2, error) {

	var tasks []*TaskV2

	for _, pending := range pendingNonces {

		isRelayed, err := s.isNonceRelayed(ctx, uint64(pending.Nonce))
		if err != nil {
			return nil, fmt.Errorf("check nonce relayed: %w", err)
		}
		if isRelayed {
			log.WithFields(log.Fields{
				"nonce": uint64(pending.Nonce),
			}).Debug("already relayed, just skip")
			continue
		}

		messagesKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "EthereumOutboundQueueV2", "Messages", nil, nil)
		if err != nil {
			return nil, fmt.Errorf("create storage key: %w", err)
		}

		currentBlockNumber := uint64(pending.BlockNumber)

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

		var messages []OutboundQueueMessageV2
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
			m := OutboundQueueMessageV2{}
			err = decoder.Decode(&m)
			if err != nil {
				return nil, fmt.Errorf("decode message error: %w", err)
			}
			messages = append(messages, m)
		}

		// For the outbound channel, the commitment hash is the merkle root of the messages
		// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/parachain/pallets/basic-channel/src/outbound/mod.rs#L275-L277
		// To verify it we fetch the message proof from the parachain
		result, err := scanForOutboundQueueProofs(
			s.paraConn.API(),
			blockHash,
			*commitmentHash,
			messages,
		)
		if err != nil {
			return nil, err
		}

		if len(result.proofs) > 0 {
			task := TaskV2{
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

type PersistedValidationData struct {
	ParentHead             []byte
	RelayParentNumber      uint32
	RelayParentStorageRoot types.Hash
	MaxPOVSize             uint32
}

// For each task, gatherProofInputs will search to find the relay chain block
// in which that header was included as well as the parachain heads for that block.
func (s *Scanner) gatherProofInputs(
	tasks []*TaskV2,
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

// The process for finalizing a backed parachain header times out after these many blocks:
const FinalizationTimeout = 4

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

	var validationData PersistedValidationData
	ok, err := s.paraConn.API().RPC.State.GetStorage(validationDataKey, &validationData, paraBlockHash)
	if err != nil {
		return 0, fmt.Errorf("fetch PersistedValidationData for block %v: %w", paraBlockHash.Hex(), err)
	}
	if !ok {
		return 0, fmt.Errorf("PersistedValidationData not found for block %v", paraBlockHash.Hex())
	}

	startBlock := validationData.RelayParentNumber + 1
	for i := validationData.RelayParentNumber + 1; i < startBlock+FinalizationTimeout; i++ {
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

func scanForOutboundQueueProofs(
	api *gsrpc.SubstrateAPI,
	blockHash types.Hash,
	commitmentHash types.H256,
	messages []OutboundQueueMessageV2,
) (*struct {
	proofs []MessageProofV2
}, error) {
	proofs := []MessageProofV2{}

	for i := len(messages) - 1; i >= 0; i-- {
		message := messages[i]

		messageProof, err := fetchMessageProof(api, blockHash, uint64(i), message)
		if err != nil {
			return nil, err
		}
		// Check that the merkle root in the proof is the same as the digest hash from the header
		if messageProof.Proof.Root != commitmentHash {
			return nil, fmt.Errorf(
				"Halting scan Outbound queue proof root '%v' doesn't match digest item's commitment hash '%v'",
				messageProof.Proof.Root,
				commitmentHash,
			)
		}

		// Collect these commitments
		proofs = append(proofs, messageProof)
	}

	return &struct {
		proofs []MessageProofV2
	}{
		proofs: proofs,
	}, nil
}

func fetchMessageProof(
	api *gsrpc.SubstrateAPI,
	blockHash types.Hash,
	messageIndex uint64,
	message OutboundQueueMessageV2,
) (MessageProofV2, error) {
	var proofHex string
	var proof MessageProofV2

	params, err := types.EncodeToHexString(messageIndex)
	if err != nil {
		return proof, fmt.Errorf("encode params: %w", err)
	}

	err = api.Client.Call(&proofHex, "state_call", "OutboundQueueApiV2_prove_message", params, blockHash.Hex())
	if err != nil {
		return proof, fmt.Errorf("call RPC OutboundQueueApi_prove_message(%v, %v): %w", messageIndex, blockHash, err)
	}

	var optionRawMerkleProof OptionRawMerkleProof
	err = types.DecodeFromHexString(proofHex, &optionRawMerkleProof)
	if err != nil {
		return proof, fmt.Errorf("decode merkle proof: %w", err)
	}

	if !optionRawMerkleProof.HasValue {
		return proof, fmt.Errorf("retrieve proof failed")
	}

	merkleProof, err := NewMerkleProof(optionRawMerkleProof.Value)
	if err != nil {
		return proof, fmt.Errorf("decode merkle proof: %w", err)
	}

	return MessageProofV2{Message: message, Proof: merkleProof}, nil
}

func (s *Scanner) isNonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	var isRelayed bool
	gatewayAddress := common.HexToAddress(s.config.Contracts.Gateway)
	gatewayContract, err := contracts.NewGateway(
		gatewayAddress,
		s.ethConn.Client(),
	)
	if err != nil {
		return isRelayed, fmt.Errorf("create gateway contract for address '%v': %w", gatewayAddress, err)
	}

	options := bind.CallOpts{
		Pending: true,
		Context: ctx,
	}
	isRelayed, err = gatewayContract.V2IsDispatched(&options, nonce)
	if err != nil {
		return isRelayed, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	return isRelayed, nil
}

func (s *Scanner) findOrderUndelivered(
	ctx context.Context,
) ([]*PendingOrder, error) {
	storageKey, err := types.CreateStorageKey(s.paraConn.Metadata(), "EthereumOutboundQueueV2", "PendingOrders", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key for parachain outbound queue PendingOrders: %w", err)
	}
	keys, err := s.paraConn.API().RPC.State.GetKeysLatest(storageKey)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v': %w", storageKey, err)
	}
	var undeliveredOrders []*PendingOrder
	for _, key := range keys {
		var undeliveredOrder PendingOrder
		value, err := s.paraConn.API().RPC.State.GetStorageRawLatest(key)
		if err != nil {
			return nil, fmt.Errorf("fetch value of pendingOrder with key '%v': %w", key, err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*value))
		err = decoder.Decode(&undeliveredOrder)
		if err != nil {
			return nil, fmt.Errorf("decode order error: %w", err)
		}
		isRelayed, err := s.isNonceRelayed(ctx, uint64(undeliveredOrder.Nonce))
		if err != nil {
			return nil, fmt.Errorf("check nonce relayed: %w", err)
		}
		if isRelayed {
			log.WithFields(log.Fields{
				"nonce": uint64(undeliveredOrder.Nonce),
			}).Debug("Relayed but not delivered to BH")
			undeliveredOrders = append(undeliveredOrders, &undeliveredOrder)
		}
	}

	return undeliveredOrders, nil
}
