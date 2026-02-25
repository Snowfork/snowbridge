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
	inactivityScoresHash               [32]byte // Hash (~8MB saved)
	latestExecutionPayloadHeaderHash  [32]byte // Hash of execution payload header
	nextWithdrawalIndex               uint64
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

	// For Fulu fork only
	proposerLookaheadHash [32]byte
	isFuluState           bool
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

	// Fulu-specific offsets (after Electra fields)
	offsetProposerLookahead = 2736713
	offsetProposerLookaheadEnd = 2737225 // 2736713 + 64*8
	minStateSizeFulu        = 2737225
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
	s.historicalRootsHash = hashHistoricalRoots(buf[o7:o9])

	// Field 8: Eth1Data
	s.eth1Data = new(state.Eth1Data)
	if err := s.eth1Data.UnmarshalSSZ(buf[offsetEth1Data : offsetEth1Data+72]); err != nil {
		return nil, fmt.Errorf("unmarshal eth1 data: %w", err)
	}

	// Field 9: Eth1DataVotes - compute hash
	s.eth1DataVotesHash = hashEth1DataVotes(buf[o9:o11])

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

	// Field 24: LatestExecutionPayloadHeader - unmarshal and compute hash
	execHeader := new(state.ExecutionPayloadHeaderDeneb)
	if err := execHeader.UnmarshalSSZ(buf[o24:o27]); err != nil {
		return nil, fmt.Errorf("unmarshal execution payload header: %w", err)
	}
	execHash, err := execHeader.HashTreeRoot()
	if err != nil {
		return nil, fmt.Errorf("hash execution payload header: %w", err)
	}
	s.latestExecutionPayloadHeaderHash = execHash

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
	s.historicalRootsHash = hashHistoricalRoots(buf[o7:o9])

	s.eth1Data = new(state.Eth1Data)
	if err := s.eth1Data.UnmarshalSSZ(buf[offsetEth1Data : offsetEth1Data+72]); err != nil {
		return nil, fmt.Errorf("unmarshal eth1 data: %w", err)
	}

	s.eth1DataVotesHash = hashEth1DataVotes(buf[o9:o11])
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

	// Field 24: LatestExecutionPayloadHeader - unmarshal and compute hash
	execHeader := new(state.ExecutionPayloadHeaderDeneb)
	if err := execHeader.UnmarshalSSZ(buf[o24:o27]); err != nil {
		return nil, fmt.Errorf("unmarshal execution payload header: %w", err)
	}
	execHash, err := execHeader.HashTreeRoot()
	if err != nil {
		return nil, fmt.Errorf("hash execution payload header: %w", err)
	}
	s.latestExecutionPayloadHeaderHash = execHash

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

// UnmarshalSSZLiteFulu unmarshals a Fulu beacon state, extracting only the fields
// needed for proof generation and computing hashes for the rest.
// Fulu adds ProposerLookahead (field 37) compared to Electra.
func UnmarshalSSZLiteFulu(buf []byte) (*LiteBeaconState, error) {
	size := uint64(len(buf))
	if size < minStateSizeFulu {
		return nil, fmt.Errorf("buffer too small for Fulu beacon state: %d < %d", size, minStateSizeFulu)
	}

	// Parse as Electra first (shares most structure)
	s, err := UnmarshalSSZLiteElectra(buf)
	if err != nil {
		return nil, fmt.Errorf("unmarshal electra base: %w", err)
	}

	// Add Fulu-specific field: ProposerLookahead (64 uint64s = 512 bytes)
	// This is a fixed-size vector, so we hash it directly
	s.proposerLookaheadHash = hashProposerLookahead(buf[offsetProposerLookahead:offsetProposerLookaheadEnd])
	s.isFuluState = true

	return s, nil
}

// UnmarshalSSZ implements the BeaconState interface but is not used for LiteBeaconState.
// Use UnmarshalSSZLiteDeneb, UnmarshalSSZLiteElectra, or UnmarshalSSZLiteFulu instead.
func (s *LiteBeaconState) UnmarshalSSZ(buf []byte) error {
	return fmt.Errorf("UnmarshalSSZ not supported on LiteBeaconState; use UnmarshalSSZLiteDeneb, UnmarshalSSZLiteElectra, or UnmarshalSSZLiteFulu")
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
	return ssz.ProofTree(s)
}

// HashTreeRoot returns the hash tree root of the lite beacon state
func (s *LiteBeaconState) HashTreeRoot() ([32]byte, error) {
	return ssz.HashWithDefaultHasher(s)
}

// HashTreeRootWith implements the ssz.HashRoot interface.
// This is the key method that enables memory-efficient proof generation by using
// pre-computed hashes for large fields (Validators, Balances, etc.) instead of
// iterating through all the data.
func (s *LiteBeaconState) HashTreeRootWith(hh ssz.HashWalker) (err error) {
	indx := hh.Index()

	// Field (0) 'GenesisTime'
	hh.PutUint64(s.genesisTime)

	// Field (1) 'GenesisValidatorsRoot'
	hh.PutBytes(s.genesisValidatorsRoot[:])

	// Field (2) 'Slot'
	hh.PutUint64(s.Slot)

	// Field (3) 'Fork'
	if err = s.fork.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (4) 'LatestBlockHeader'
	if err = s.LatestBlockHeader.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (5) 'BlockRoots' - we have full data
	{
		subIndx := hh.Index()
		for _, i := range s.BlockRoots {
			hh.Append(i)
		}
		hh.Merkleize(subIndx)
	}

	// Field (6) 'StateRoots' - use pre-computed hash
	hh.PutBytes(s.stateRootsHash[:])

	// Field (7) 'HistoricalRoots' - use pre-computed hash
	hh.PutBytes(s.historicalRootsHash[:])

	// Field (8) 'Eth1Data'
	if err = s.eth1Data.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (9) 'Eth1DataVotes' - use pre-computed hash
	hh.PutBytes(s.eth1DataVotesHash[:])

	// Field (10) 'Eth1DepositIndex'
	hh.PutUint64(s.eth1DepositIndex)

	// Field (11) 'Validators' - use pre-computed hash (HUGE MEMORY SAVINGS)
	hh.PutBytes(s.validatorsHash[:])

	// Field (12) 'Balances' - use pre-computed hash
	hh.PutBytes(s.balancesHash[:])

	// Field (13) 'RandaoMixes' - use pre-computed hash
	hh.PutBytes(s.randaoMixesHash[:])

	// Field (14) 'Slashings' - use pre-computed hash
	hh.PutBytes(s.slashingsHash[:])

	// Field (15) 'PreviousEpochParticipation' - use pre-computed hash
	hh.PutBytes(s.previousEpochParticipationHash[:])

	// Field (16) 'CurrentEpochParticipation' - use pre-computed hash
	hh.PutBytes(s.currentEpochParticipationHash[:])

	// Field (17) 'JustificationBits'
	hh.PutBytes(s.justificationBits)

	// Field (18) 'PreviousJustifiedCheckpoint'
	if err = s.previousJustifiedCheckpoint.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (19) 'CurrentJustifiedCheckpoint'
	if err = s.currentJustifiedCheckpoint.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (20) 'FinalizedCheckpoint'
	if err = s.FinalizedCheckpoint.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (21) 'InactivityScores' - use pre-computed hash
	hh.PutBytes(s.inactivityScoresHash[:])

	// Field (22) 'CurrentSyncCommittee'
	if err = s.CurrentSyncCommittee.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (23) 'NextSyncCommittee'
	if err = s.NextSyncCommittee.HashTreeRootWith(hh); err != nil {
		return
	}

	// Field (24) 'LatestExecutionPayloadHeader' - use pre-computed hash
	hh.PutBytes(s.latestExecutionPayloadHeaderHash[:])

	// Field (25) 'NextWithdrawalIndex'
	hh.PutUint64(s.nextWithdrawalIndex)

	// Field (26) 'NextWithdrawalValidatorIndex'
	hh.PutUint64(s.nextWithdrawalValidatorIndex)

	// Field (27) 'HistoricalSummaries' - use pre-computed hash
	hh.PutBytes(s.historicalSummariesHash[:])

	// Electra/Fulu-specific fields (28-36)
	if s.isElectra() || s.isFuluState {
		// Field (28) 'DepositRequestsStartIndex'
		hh.PutUint64(s.depositRequestsStartIndex)

		// Field (29) 'DepositBalanceToConsume'
		hh.PutUint64(s.depositBalanceToConsume)

		// Field (30) 'ExitBalanceToConsume'
		hh.PutUint64(s.exitBalanceToConsume)

		// Field (31) 'EarliestExitEpoch'
		hh.PutUint64(s.earliestExitEpoch)

		// Field (32) 'ConsolidationBalanceToConsume'
		hh.PutUint64(s.consolidationBalanceToConsume)

		// Field (33) 'EarliestConsolidationEpoch'
		hh.PutUint64(s.earliestConsolidationEpoch)

		// Field (34) 'PendingDeposits' - use pre-computed hash
		hh.PutBytes(s.pendingDepositsHash[:])

		// Field (35) 'PendingPartialWithdrawals' - use pre-computed hash
		hh.PutBytes(s.pendingPartialWithdrawalsHash[:])

		// Field (36) 'PendingConsolidations' - use pre-computed hash
		hh.PutBytes(s.pendingConsolidationsHash[:])

		// Fulu-specific field
		if s.isFuluState {
			// Field (37) 'ProposerLookahead' - use pre-computed hash
			hh.PutBytes(s.proposerLookaheadHash[:])
		}
	}

	hh.Merkleize(indx)
	return
}

// isElectra returns true if this is an Electra state
func (s *LiteBeaconState) isElectra() bool {
	return s.depositRequestsStartIndex != 0 || s.pendingDepositsHash != [32]byte{}
}

