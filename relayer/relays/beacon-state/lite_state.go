package beaconstate

import (
	"encoding/binary"
	"fmt"

	ssz "github.com/ferranbt/fastssz"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// LiteBeaconState contains only the fields needed for proof generation.
// Large fields (Validators, Balances, etc.) are represented only by their
// hash tree roots, saving ~130MB+ of memory per state.
type LiteBeaconState struct {
	// Fields we need for proofs (actual data)
	Slot                 uint64
	LatestBlockHeader    *state.BeaconBlockHeader
	BlockRoots           [][]byte // 8192 x 32 bytes = 256KB
	FinalizedCheckpoint  *state.Checkpoint
	CurrentSyncCommittee *state.SyncCommittee
	NextSyncCommittee    *state.SyncCommittee

	// Hashes of fields we don't need data for (for tree construction)
	genesisTime                     uint64
	genesisValidatorsRoot           [32]byte
	fork                            *state.Fork
	stateRootsHash                  [32]byte // Hash of StateRoots
	historicalRootsHash             [32]byte
	eth1Data                        *state.Eth1Data
	eth1DataVotesHash               [32]byte
	eth1DepositIndex                uint64
	validatorsHash                  [32]byte // Hash of Validators (~120MB saved)
	balancesHash                    [32]byte // Hash of Balances (~8MB saved)
	randaoMixesHash                 [32]byte
	slashingsHash                   [32]byte
	previousEpochParticipationHash  [32]byte // Hash (~1MB saved)
	currentEpochParticipationHash   [32]byte // Hash (~1MB saved)
	justificationBits               []byte
	previousJustifiedCheckpoint     *state.Checkpoint
	currentJustifiedCheckpoint      *state.Checkpoint
	inactivityScoresHash            [32]byte // Hash (~8MB saved)
	latestExecutionPayloadHeader    []byte   // Keep raw for hash
	nextWithdrawalIndex             uint64
	nextWithdrawalValidatorIndex    uint64
	historicalSummariesHash         [32]byte

	// For Electra/Fulu forks
	pendingDepositsHash             [32]byte
	pendingPartialWithdrawalsHash   [32]byte
	pendingConsolidationsHash       [32]byte
	depositRequestsStartIndex       uint64
	depositBalanceToConsume         uint64
	exitBalanceToConsume            uint64
	earliestExitEpoch               uint64
	consolidationBalanceToConsume   uint64
	earliestConsolidationEpoch      uint64
}

// SSZ byte offsets for BeaconState fixed part (shared by Deneb, Electra, Fulu)
const (
	offsetGenesisTime           = 0
	offsetGenesisValidatorsRoot = 8
	offsetSlot                  = 40
	offsetFork                  = 48
	offsetLatestBlockHeader     = 64
	offsetBlockRoots            = 176
	offsetBlockRootsEnd         = 262320
	offsetStateRoots            = 262320
	offsetStateRootsEnd         = 524464
	offsetHistoricalRootsPtr    = 524464
	offsetEth1Data              = 524468
	offsetEth1DataVotesPtr      = 524540
	offsetEth1DepositIndex      = 524544
	offsetValidatorsPtr         = 524552
	offsetBalancesPtr           = 524556
	offsetRandaoMixes           = 524560
	offsetRandaoMixesEnd        = 2621712
	offsetSlashings             = 2621712
	offsetSlashingsEnd          = 2687248
	offsetPrevEpochPartPtr      = 2687248
	offsetCurrEpochPartPtr      = 2687252
	offsetJustificationBits     = 2687256
	offsetPrevJustifiedCkpt     = 2687257
	offsetCurrJustifiedCkpt     = 2687297
	offsetFinalizedCheckpoint   = 2687337
	offsetInactivityScoresPtr   = 2687377
	offsetCurrentSyncCommittee  = 2687381
	offsetNextSyncCommittee     = 2712005
	offsetExecPayloadHeaderPtr  = 2736629
	offsetNextWithdrawalIndex   = 2736633
	offsetNextWithdrawalValIdx  = 2736641
	offsetHistoricalSummPtr     = 2736649
	minStateSizeDeneb           = 2736653

	// Electra-specific offsets (after Deneb fields)
	offsetDepositRequestsStartIndex     = 2736653
	offsetDepositBalanceToConsume       = 2736661
	offsetExitBalanceToConsume          = 2736669
	offsetEarliestExitEpoch             = 2736677
	offsetConsolidationBalanceToConsume = 2736685
	offsetEarliestConsolidationEpoch    = 2736693
	offsetPendingDepositsPtr            = 2736701
	offsetPendingPartialWithdrawalsPtr  = 2736705
	offsetPendingConsolidationsPtr      = 2736709
	minStateSizeElectra                 = 2736713
)

// UnmarshalSSZLite unmarshals a Deneb beacon state, extracting only the fields
// needed for proof generation and computing hashes for the rest.
// This saves ~130MB+ of memory compared to full unmarshaling.
func UnmarshalSSZLiteDeneb(buf []byte) (*LiteBeaconState, error) {
	size := uint64(len(buf))
	if size < minStateSizeDeneb {
		return nil, fmt.Errorf("buffer too small for beacon state: %d < %d", size, minStateSizeDeneb)
	}

	s := &LiteBeaconState{}

	// Read variable-length field offsets
	o7 := binary.LittleEndian.Uint32(buf[offsetHistoricalRootsPtr:])  // HistoricalRoots
	o9 := binary.LittleEndian.Uint32(buf[offsetEth1DataVotesPtr:])    // Eth1DataVotes
	o11 := binary.LittleEndian.Uint32(buf[offsetValidatorsPtr:])      // Validators
	o12 := binary.LittleEndian.Uint32(buf[offsetBalancesPtr:])        // Balances
	o15 := binary.LittleEndian.Uint32(buf[offsetPrevEpochPartPtr:])   // PreviousEpochParticipation
	o16 := binary.LittleEndian.Uint32(buf[offsetCurrEpochPartPtr:])   // CurrentEpochParticipation
	o21 := binary.LittleEndian.Uint32(buf[offsetInactivityScoresPtr:])// InactivityScores
	o24 := binary.LittleEndian.Uint32(buf[offsetExecPayloadHeaderPtr:])// LatestExecutionPayloadHeader
	o27 := binary.LittleEndian.Uint32(buf[offsetHistoricalSummPtr:])  // HistoricalSummaries

	// === Fields we need (extract data) ===

	// Field 2: Slot
	s.Slot = binary.LittleEndian.Uint64(buf[offsetSlot:])

	// Field 4: LatestBlockHeader
	s.LatestBlockHeader = new(state.BeaconBlockHeader)
	if err := s.LatestBlockHeader.UnmarshalSSZ(buf[offsetLatestBlockHeader : offsetLatestBlockHeader+112]); err != nil {
		return nil, fmt.Errorf("unmarshal latest block header: %w", err)
	}

	// Field 5: BlockRoots (256KB - we need this for block root proofs)
	s.BlockRoots = make([][]byte, 8192)
	for i := 0; i < 8192; i++ {
		s.BlockRoots[i] = make([]byte, 32)
		copy(s.BlockRoots[i], buf[offsetBlockRoots+i*32:])
	}

	// Field 20: FinalizedCheckpoint
	s.FinalizedCheckpoint = new(state.Checkpoint)
	if err := s.FinalizedCheckpoint.UnmarshalSSZ(buf[offsetFinalizedCheckpoint : offsetFinalizedCheckpoint+40]); err != nil {
		return nil, fmt.Errorf("unmarshal finalized checkpoint: %w", err)
	}

	// Field 22: CurrentSyncCommittee
	s.CurrentSyncCommittee = new(state.SyncCommittee)
	if err := s.CurrentSyncCommittee.UnmarshalSSZ(buf[offsetCurrentSyncCommittee : offsetCurrentSyncCommittee+24624]); err != nil {
		return nil, fmt.Errorf("unmarshal current sync committee: %w", err)
	}

	// Field 23: NextSyncCommittee
	s.NextSyncCommittee = new(state.SyncCommittee)
	if err := s.NextSyncCommittee.UnmarshalSSZ(buf[offsetNextSyncCommittee : offsetNextSyncCommittee+24624]); err != nil {
		return nil, fmt.Errorf("unmarshal next sync committee: %w", err)
	}

	// === Fields we hash only (don't store data) ===

	// Field 0: GenesisTime (small, just store)
	s.genesisTime = binary.LittleEndian.Uint64(buf[offsetGenesisTime:])

	// Field 1: GenesisValidatorsRoot
	copy(s.genesisValidatorsRoot[:], buf[offsetGenesisValidatorsRoot:offsetGenesisValidatorsRoot+32])

	// Field 3: Fork
	s.fork = new(state.Fork)
	if err := s.fork.UnmarshalSSZ(buf[offsetFork : offsetFork+16]); err != nil {
		return nil, fmt.Errorf("unmarshal fork: %w", err)
	}

	// Field 6: StateRoots - compute hash
	s.stateRootsHash = hashFixedVector(buf[offsetStateRoots:offsetStateRootsEnd], 32, 8192)

	// Field 7: HistoricalRoots - compute hash
	s.historicalRootsHash = hashVariableList(buf[o7:o9], 32)

	// Field 8: Eth1Data
	s.eth1Data = new(state.Eth1Data)
	if err := s.eth1Data.UnmarshalSSZ(buf[offsetEth1Data : offsetEth1Data+72]); err != nil {
		return nil, fmt.Errorf("unmarshal eth1 data: %w", err)
	}

	// Field 9: Eth1DataVotes - compute hash
	s.eth1DataVotesHash = hashVariableList(buf[o9:o11], 72)

	// Field 10: Eth1DepositIndex
	s.eth1DepositIndex = binary.LittleEndian.Uint64(buf[offsetEth1DepositIndex:])

	// Field 11: Validators - compute hash (HUGE SAVINGS: ~120MB)
	s.validatorsHash = hashValidators(buf[o11:o12])

	// Field 12: Balances - compute hash (SAVINGS: ~8MB)
	s.balancesHash = hashBalances(buf[o12:o15])

	// Field 13: RandaoMixes - compute hash
	s.randaoMixesHash = hashFixedVector(buf[offsetRandaoMixes:offsetRandaoMixesEnd], 32, 65536)

	// Field 14: Slashings - compute hash
	s.slashingsHash = hashSlashings(buf[offsetSlashings:offsetSlashingsEnd])

	// Field 15: PreviousEpochParticipation - compute hash
	s.previousEpochParticipationHash = hashParticipation(buf[o15:o16])

	// Field 16: CurrentEpochParticipation - compute hash
	s.currentEpochParticipationHash = hashParticipation(buf[o16:o21])

	// Field 17: JustificationBits
	s.justificationBits = make([]byte, 1)
	copy(s.justificationBits, buf[offsetJustificationBits:offsetJustificationBits+1])

	// Field 18: PreviousJustifiedCheckpoint
	s.previousJustifiedCheckpoint = new(state.Checkpoint)
	if err := s.previousJustifiedCheckpoint.UnmarshalSSZ(buf[offsetPrevJustifiedCkpt : offsetPrevJustifiedCkpt+40]); err != nil {
		return nil, fmt.Errorf("unmarshal previous justified checkpoint: %w", err)
	}

	// Field 19: CurrentJustifiedCheckpoint
	s.currentJustifiedCheckpoint = new(state.Checkpoint)
	if err := s.currentJustifiedCheckpoint.UnmarshalSSZ(buf[offsetCurrJustifiedCkpt : offsetCurrJustifiedCkpt+40]); err != nil {
		return nil, fmt.Errorf("unmarshal current justified checkpoint: %w", err)
	}

	// Field 21: InactivityScores - compute hash (SAVINGS: ~8MB)
	s.inactivityScoresHash = hashInactivityScores(buf[o21:o24])

	// Field 24: LatestExecutionPayloadHeader - store raw for hashing
	s.latestExecutionPayloadHeader = make([]byte, len(buf[o24:o27]))
	copy(s.latestExecutionPayloadHeader, buf[o24:o27])

	// Field 25: NextWithdrawalIndex
	s.nextWithdrawalIndex = binary.LittleEndian.Uint64(buf[offsetNextWithdrawalIndex:])

	// Field 26: NextWithdrawalValidatorIndex
	s.nextWithdrawalValidatorIndex = binary.LittleEndian.Uint64(buf[offsetNextWithdrawalValIdx:])

	// Field 27: HistoricalSummaries - compute hash
	s.historicalSummariesHash = hashHistoricalSummaries(buf[o27:])

	return s, nil
}

// UnmarshalSSZLiteElectra unmarshals an Electra beacon state, extracting only the fields
// needed for proof generation and computing hashes for the rest.
func UnmarshalSSZLiteElectra(buf []byte) (*LiteBeaconState, error) {
	size := uint64(len(buf))
	if size < minStateSizeElectra {
		return nil, fmt.Errorf("buffer too small for Electra beacon state: %d < %d", size, minStateSizeElectra)
	}

	// Electra shares most structure with Deneb, so we start with Deneb parsing
	// but with adjusted offsets for the extra fields
	s := &LiteBeaconState{}

	// Read variable-length field offsets
	o7 := binary.LittleEndian.Uint32(buf[offsetHistoricalRootsPtr:])
	o9 := binary.LittleEndian.Uint32(buf[offsetEth1DataVotesPtr:])
	o11 := binary.LittleEndian.Uint32(buf[offsetValidatorsPtr:])
	o12 := binary.LittleEndian.Uint32(buf[offsetBalancesPtr:])
	o15 := binary.LittleEndian.Uint32(buf[offsetPrevEpochPartPtr:])
	o16 := binary.LittleEndian.Uint32(buf[offsetCurrEpochPartPtr:])
	o21 := binary.LittleEndian.Uint32(buf[offsetInactivityScoresPtr:])
	o24 := binary.LittleEndian.Uint32(buf[offsetExecPayloadHeaderPtr:])
	o27 := binary.LittleEndian.Uint32(buf[offsetHistoricalSummPtr:])
	o34 := binary.LittleEndian.Uint32(buf[offsetPendingDepositsPtr:])
	o35 := binary.LittleEndian.Uint32(buf[offsetPendingPartialWithdrawalsPtr:])
	o36 := binary.LittleEndian.Uint32(buf[offsetPendingConsolidationsPtr:])

	// === Fields we need (extract data) ===

	// Field 2: Slot
	s.Slot = binary.LittleEndian.Uint64(buf[offsetSlot:])

	// Field 4: LatestBlockHeader
	s.LatestBlockHeader = new(state.BeaconBlockHeader)
	if err := s.LatestBlockHeader.UnmarshalSSZ(buf[offsetLatestBlockHeader : offsetLatestBlockHeader+112]); err != nil {
		return nil, fmt.Errorf("unmarshal latest block header: %w", err)
	}

	// Field 5: BlockRoots
	s.BlockRoots = make([][]byte, 8192)
	for i := 0; i < 8192; i++ {
		s.BlockRoots[i] = make([]byte, 32)
		copy(s.BlockRoots[i], buf[offsetBlockRoots+i*32:])
	}

	// Field 20: FinalizedCheckpoint
	s.FinalizedCheckpoint = new(state.Checkpoint)
	if err := s.FinalizedCheckpoint.UnmarshalSSZ(buf[offsetFinalizedCheckpoint : offsetFinalizedCheckpoint+40]); err != nil {
		return nil, fmt.Errorf("unmarshal finalized checkpoint: %w", err)
	}

	// Field 22: CurrentSyncCommittee
	s.CurrentSyncCommittee = new(state.SyncCommittee)
	if err := s.CurrentSyncCommittee.UnmarshalSSZ(buf[offsetCurrentSyncCommittee : offsetCurrentSyncCommittee+24624]); err != nil {
		return nil, fmt.Errorf("unmarshal current sync committee: %w", err)
	}

	// Field 23: NextSyncCommittee
	s.NextSyncCommittee = new(state.SyncCommittee)
	if err := s.NextSyncCommittee.UnmarshalSSZ(buf[offsetNextSyncCommittee : offsetNextSyncCommittee+24624]); err != nil {
		return nil, fmt.Errorf("unmarshal next sync committee: %w", err)
	}

	// === Fields we hash only ===

	s.genesisTime = binary.LittleEndian.Uint64(buf[offsetGenesisTime:])
	copy(s.genesisValidatorsRoot[:], buf[offsetGenesisValidatorsRoot:offsetGenesisValidatorsRoot+32])

	s.fork = new(state.Fork)
	if err := s.fork.UnmarshalSSZ(buf[offsetFork : offsetFork+16]); err != nil {
		return nil, fmt.Errorf("unmarshal fork: %w", err)
	}

	s.stateRootsHash = hashFixedVector(buf[offsetStateRoots:offsetStateRootsEnd], 32, 8192)
	s.historicalRootsHash = hashVariableList(buf[o7:o9], 32)

	s.eth1Data = new(state.Eth1Data)
	if err := s.eth1Data.UnmarshalSSZ(buf[offsetEth1Data : offsetEth1Data+72]); err != nil {
		return nil, fmt.Errorf("unmarshal eth1 data: %w", err)
	}

	s.eth1DataVotesHash = hashVariableList(buf[o9:o11], 72)
	s.eth1DepositIndex = binary.LittleEndian.Uint64(buf[offsetEth1DepositIndex:])
	s.validatorsHash = hashValidators(buf[o11:o12])
	s.balancesHash = hashBalances(buf[o12:o15])
	s.randaoMixesHash = hashFixedVector(buf[offsetRandaoMixes:offsetRandaoMixesEnd], 32, 65536)
	s.slashingsHash = hashSlashings(buf[offsetSlashings:offsetSlashingsEnd])
	s.previousEpochParticipationHash = hashParticipation(buf[o15:o16])
	s.currentEpochParticipationHash = hashParticipation(buf[o16:o21])

	s.justificationBits = make([]byte, 1)
	copy(s.justificationBits, buf[offsetJustificationBits:offsetJustificationBits+1])

	s.previousJustifiedCheckpoint = new(state.Checkpoint)
	if err := s.previousJustifiedCheckpoint.UnmarshalSSZ(buf[offsetPrevJustifiedCkpt : offsetPrevJustifiedCkpt+40]); err != nil {
		return nil, fmt.Errorf("unmarshal previous justified checkpoint: %w", err)
	}

	s.currentJustifiedCheckpoint = new(state.Checkpoint)
	if err := s.currentJustifiedCheckpoint.UnmarshalSSZ(buf[offsetCurrJustifiedCkpt : offsetCurrJustifiedCkpt+40]); err != nil {
		return nil, fmt.Errorf("unmarshal current justified checkpoint: %w", err)
	}

	s.inactivityScoresHash = hashInactivityScores(buf[o21:o24])
	s.latestExecutionPayloadHeader = make([]byte, len(buf[o24:o27]))
	copy(s.latestExecutionPayloadHeader, buf[o24:o27])

	s.nextWithdrawalIndex = binary.LittleEndian.Uint64(buf[offsetNextWithdrawalIndex:])
	s.nextWithdrawalValidatorIndex = binary.LittleEndian.Uint64(buf[offsetNextWithdrawalValIdx:])
	s.historicalSummariesHash = hashHistoricalSummaries(buf[o27:o34])

	// Electra-specific fields
	s.depositRequestsStartIndex = binary.LittleEndian.Uint64(buf[offsetDepositRequestsStartIndex:])
	s.depositBalanceToConsume = binary.LittleEndian.Uint64(buf[offsetDepositBalanceToConsume:])
	s.exitBalanceToConsume = binary.LittleEndian.Uint64(buf[offsetExitBalanceToConsume:])
	s.earliestExitEpoch = binary.LittleEndian.Uint64(buf[offsetEarliestExitEpoch:])
	s.consolidationBalanceToConsume = binary.LittleEndian.Uint64(buf[offsetConsolidationBalanceToConsume:])
	s.earliestConsolidationEpoch = binary.LittleEndian.Uint64(buf[offsetEarliestConsolidationEpoch:])

	// Hash the new variable-length fields
	s.pendingDepositsHash = hashPendingDeposits(buf[o34:o35])
	s.pendingPartialWithdrawalsHash = hashPendingPartialWithdrawals(buf[o35:o36])
	s.pendingConsolidationsHash = hashPendingConsolidations(buf[o36:])

	return s, nil
}

// UnmarshalSSZ implements the BeaconState interface but is not used for LiteBeaconState.
// Use UnmarshalSSZLiteDeneb or UnmarshalSSZLiteElectra instead.
func (s *LiteBeaconState) UnmarshalSSZ(buf []byte) error {
	return fmt.Errorf("UnmarshalSSZ not supported on LiteBeaconState; use UnmarshalSSZLiteDeneb or UnmarshalSSZLiteElectra")
}

// GetSlot returns the slot of this beacon state
func (s *LiteBeaconState) GetSlot() uint64 {
	return s.Slot
}

// GetLatestBlockHeader returns the latest block header
func (s *LiteBeaconState) GetLatestBlockHeader() *state.BeaconBlockHeader {
	return s.LatestBlockHeader
}

// GetBlockRoots returns the block roots array
func (s *LiteBeaconState) GetBlockRoots() [][]byte {
	return s.BlockRoots
}

// GetFinalizedCheckpoint returns the finalized checkpoint
func (s *LiteBeaconState) GetFinalizedCheckpoint() *state.Checkpoint {
	return s.FinalizedCheckpoint
}

// GetCurrentSyncCommittee returns the current sync committee
func (s *LiteBeaconState) GetCurrentSyncCommittee() *state.SyncCommittee {
	return s.CurrentSyncCommittee
}

// GetNextSyncCommittee returns the next sync committee
func (s *LiteBeaconState) GetNextSyncCommittee() *state.SyncCommittee {
	return s.NextSyncCommittee
}

// GetTree builds an SSZ Merkle tree for this lite state.
// This uses pre-computed hashes for large fields, saving significant memory.
func (s *LiteBeaconState) GetTree() (*ssz.Node, error) {
	return s.buildMerkleTree()
}

// buildMerkleTree constructs the SSZ Merkle tree using pre-computed hashes
func (s *LiteBeaconState) buildMerkleTree() (*ssz.Node, error) {
	// Determine if this is Electra (has additional fields)
	isElectra := s.depositRequestsStartIndex != 0 || s.pendingDepositsHash != [32]byte{}

	// Build leaf nodes for each field
	numFields := 28 // Deneb has 28 fields (indices 0-27)
	if isElectra {
		numFields = 37 // Electra has 37 fields (indices 0-36)
	}
	leaves := make([]*ssz.Node, numFields)

	// Field 0: GenesisTime
	leaves[0] = ssz.LeafFromUint64(s.genesisTime)

	// Field 1: GenesisValidatorsRoot
	leaves[1] = ssz.LeafFromBytes(s.genesisValidatorsRoot[:])

	// Field 2: Slot
	leaves[2] = ssz.LeafFromUint64(s.Slot)

	// Field 3: Fork
	forkNode, err := s.fork.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get fork tree: %w", err)
	}
	leaves[3] = forkNode

	// Field 4: LatestBlockHeader
	headerNode, err := s.LatestBlockHeader.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get header tree: %w", err)
	}
	leaves[4] = headerNode

	// Field 5: BlockRoots (we have the actual data)
	blockRootsNode, err := buildBlockRootsTree(s.BlockRoots)
	if err != nil {
		return nil, fmt.Errorf("build block roots tree: %w", err)
	}
	leaves[5] = blockRootsNode

	// Field 6: StateRoots (use pre-computed hash)
	leaves[6] = ssz.LeafFromBytes(s.stateRootsHash[:])

	// Field 7: HistoricalRoots (use pre-computed hash)
	leaves[7] = ssz.LeafFromBytes(s.historicalRootsHash[:])

	// Field 8: Eth1Data
	eth1Node, err := s.eth1Data.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get eth1 data tree: %w", err)
	}
	leaves[8] = eth1Node

	// Field 9: Eth1DataVotes (use pre-computed hash)
	leaves[9] = ssz.LeafFromBytes(s.eth1DataVotesHash[:])

	// Field 10: Eth1DepositIndex
	leaves[10] = ssz.LeafFromUint64(s.eth1DepositIndex)

	// Field 11: Validators (use pre-computed hash - HUGE SAVINGS)
	leaves[11] = ssz.LeafFromBytes(s.validatorsHash[:])

	// Field 12: Balances (use pre-computed hash)
	leaves[12] = ssz.LeafFromBytes(s.balancesHash[:])

	// Field 13: RandaoMixes (use pre-computed hash)
	leaves[13] = ssz.LeafFromBytes(s.randaoMixesHash[:])

	// Field 14: Slashings (use pre-computed hash)
	leaves[14] = ssz.LeafFromBytes(s.slashingsHash[:])

	// Field 15: PreviousEpochParticipation (use pre-computed hash)
	leaves[15] = ssz.LeafFromBytes(s.previousEpochParticipationHash[:])

	// Field 16: CurrentEpochParticipation (use pre-computed hash)
	leaves[16] = ssz.LeafFromBytes(s.currentEpochParticipationHash[:])

	// Field 17: JustificationBits
	leaves[17] = ssz.LeafFromBytes(s.justificationBits)

	// Field 18: PreviousJustifiedCheckpoint
	prevJustNode, err := s.previousJustifiedCheckpoint.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get prev justified checkpoint tree: %w", err)
	}
	leaves[18] = prevJustNode

	// Field 19: CurrentJustifiedCheckpoint
	currJustNode, err := s.currentJustifiedCheckpoint.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get curr justified checkpoint tree: %w", err)
	}
	leaves[19] = currJustNode

	// Field 20: FinalizedCheckpoint
	finalizedNode, err := s.FinalizedCheckpoint.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get finalized checkpoint tree: %w", err)
	}
	leaves[20] = finalizedNode

	// Field 21: InactivityScores (use pre-computed hash)
	leaves[21] = ssz.LeafFromBytes(s.inactivityScoresHash[:])

	// Field 22: CurrentSyncCommittee
	currSyncNode, err := s.CurrentSyncCommittee.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get current sync committee tree: %w", err)
	}
	leaves[22] = currSyncNode

	// Field 23: NextSyncCommittee
	nextSyncNode, err := s.NextSyncCommittee.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get next sync committee tree: %w", err)
	}
	leaves[23] = nextSyncNode

	// Field 24: LatestExecutionPayloadHeader (hash the raw bytes)
	execPayloadHash := hashExecutionPayloadHeader(s.latestExecutionPayloadHeader)
	leaves[24] = ssz.LeafFromBytes(execPayloadHash[:])

	// Field 25: NextWithdrawalIndex
	leaves[25] = ssz.LeafFromUint64(s.nextWithdrawalIndex)

	// Field 26: NextWithdrawalValidatorIndex
	leaves[26] = ssz.LeafFromUint64(s.nextWithdrawalValidatorIndex)

	// Field 27: HistoricalSummaries (use pre-computed hash)
	leaves[27] = ssz.LeafFromBytes(s.historicalSummariesHash[:])

	// Electra-specific fields (28-36)
	if isElectra {
		leaves[28] = ssz.LeafFromUint64(s.depositRequestsStartIndex)
		leaves[29] = ssz.LeafFromUint64(s.depositBalanceToConsume)
		leaves[30] = ssz.LeafFromUint64(s.exitBalanceToConsume)
		leaves[31] = ssz.LeafFromUint64(s.earliestExitEpoch)
		leaves[32] = ssz.LeafFromUint64(s.consolidationBalanceToConsume)
		leaves[33] = ssz.LeafFromUint64(s.earliestConsolidationEpoch)
		leaves[34] = ssz.LeafFromBytes(s.pendingDepositsHash[:])
		leaves[35] = ssz.LeafFromBytes(s.pendingPartialWithdrawalsHash[:])
		leaves[36] = ssz.LeafFromBytes(s.pendingConsolidationsHash[:])
	}

	// Build the tree from leaves (pad to power of 2)
	targetSize := 32
	if isElectra {
		targetSize = 64 // Next power of 2 after 37
	}
	for len(leaves) < targetSize {
		leaves = append(leaves, ssz.LeafFromBytes(make([]byte, 32)))
	}

	return ssz.TreeFromNodes(leaves, targetSize)
}

// buildBlockRootsTree builds a Merkle tree from block roots
func buildBlockRootsTree(blockRoots [][]byte) (*ssz.Node, error) {
	leaves := make([]*ssz.Node, len(blockRoots))
	for i, root := range blockRoots {
		leaves[i] = ssz.LeafFromBytes(root)
	}
	return ssz.TreeFromNodes(leaves, 8192)
}
