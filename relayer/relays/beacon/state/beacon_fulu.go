package state

type BeaconStateFulu struct {
	GenesisTime                   uint64                       `json:"genesis_time"`
	GenesisValidatorsRoot         []byte                       `json:"genesis_validators_root" ssz-size:"32"`
	Slot                          uint64                       `json:"slot"`
	Fork                          *Fork                        `json:"fork"`
	LatestBlockHeader             *BeaconBlockHeader           `json:"latest_block_header"`
	BlockRoots                    [][]byte                     `json:"block_roots" ssz-size:"8192,32"`
	StateRoots                    [][]byte                     `json:"state_roots" ssz-size:"8192,32"`
	HistoricalRoots               [][]byte                     `json:"historical_roots" ssz-max:"16777216" ssz-size:"?,32"`
	Eth1Data                      *Eth1Data                    `json:"eth1_data"`
	Eth1DataVotes                 []*Eth1Data                  `json:"eth1_data_votes" ssz-max:"2048"`
	Eth1DepositIndex              uint64                       `json:"eth1_deposit_index"`
	Validators                    []*Validator                 `json:"validators" ssz-max:"1099511627776"`
	Balances                      []uint64                     `json:"balances" ssz-max:"1099511627776"`
	RandaoMixes                   [][]byte                     `json:"randao_mixes" ssz-size:"65536,32"`
	Slashings                     []uint64                     `json:"slashings" ssz-size:"8192"`
	PreviousEpochParticipation    []byte                       `json:"previous_epoch_participation" ssz-max:"1099511627776"`
	CurrentEpochParticipation     []byte                       `json:"current_epoch_participation" ssz-max:"1099511627776"`
	JustificationBits             []byte                       `json:"justification_bits" cast-type:"github.com/prysmaticlabs/go-bitfield.Bitvector4" ssz-size:"1"`
	PreviousJustifiedCheckpoint   *Checkpoint                  `json:"previous_justified_checkpoint"`
	CurrentJustifiedCheckpoint    *Checkpoint                  `json:"current_justified_checkpoint"`
	FinalizedCheckpoint           *Checkpoint                  `json:"finalized_checkpoint"`
	InactivityScores              []uint64                     `json:"inactivity_scores" ssz-max:"1099511627776"`
	CurrentSyncCommittee          *SyncCommittee               `json:"current_sync_committee"`
	NextSyncCommittee             *SyncCommittee               `json:"next_sync_committee"`
	LatestExecutionPayloadHeader  *ExecutionPayloadHeaderDeneb `json:"latest_execution_payload_header"`
	NextWithdrawalIndex           uint64                       `json:"next_withdrawal_index,omitempty"`
	NextWithdrawalValidatorIndex  uint64                       `json:"next_withdrawal_validator_index,omitempty"`
	HistoricalSummaries           []*HistoricalSummary         `json:"historical_summaries,omitempty" ssz-max:"16777216"`
	DepositRequestsStartIndex     uint64                       `json:"deposit_requests_start_index,omitempty"`
	DepositBalanceToConsume       uint64                       `json:"deposit_balance_to_consume,omitempty"`
	ExitBalanceToConsume          uint64                       `json:"exit_balance_to_consume,omitempty"`
	EarliestExitEpoch             uint64                       `json:"earliest_exit_epoch,omitempty"`
	ConsolidationBalanceToConsume uint64                       `json:"consolidation_balance_to_consume,omitempty"`
	EarliestConsolidationEpoch    uint64                       `json:"earliest_consolidation_epoch,omitempty"`
	PendingDeposits               []*PendingDeposit            `json:"pending_deposits,omitempty" ssz-max:"134217728"`
	PendingPartialWithdrawals     []*PendingPartialWithdrawal  `json:"pending_partial_withdrawals,omitempty" ssz-max:"134217728"`
	PendingConsolidations         []*PendingConsolidation      `json:"pending_consolidations,omitempty" ssz-max:"262144"`
	ProposerLookahead             []uint64                     `json:"proposer_lookahead,omitempty" ssz-size:"64"` // New in Fulu:EIP7917
}

func (b *BeaconStateFulu) GetSlot() uint64 {
	return b.Slot
}

func (b *BeaconStateFulu) GetLatestBlockHeader() *BeaconBlockHeader {
	return b.LatestBlockHeader
}

func (b *BeaconStateFulu) GetBlockRoots() [][]byte {
	return b.BlockRoots
}

func (b *BeaconStateFulu) SetBlockRoots(blockRoots [][]byte) {
	b.BlockRoots = blockRoots
}

func (b *BeaconStateFulu) GetFinalizedCheckpoint() *Checkpoint {
	return b.FinalizedCheckpoint
}

func (b *BeaconStateFulu) GetNextSyncCommittee() *SyncCommittee {
	return b.NextSyncCommittee
}
func (b *BeaconStateFulu) GetCurrentSyncCommittee() *SyncCommittee {
	return b.CurrentSyncCommittee
}
