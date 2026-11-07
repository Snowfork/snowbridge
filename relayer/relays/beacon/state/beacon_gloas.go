package state

// Gloas (EIP-7732) replaces latest_execution_payload_header with latest_block_hash and appends
// the builder registry, payload availability and the latest bid. 46 fields, all active.
//
// ssz-index tags make this a ProgressiveContainer for dynamic-ssz. ProgressiveList fields
// carry ssz-type:"progressive-list". Classic vectors/lists keep ssz-size / ssz-max only.
type BeaconStateGloas struct {
	GenesisTime                   uint64                       `json:"genesis_time" ssz-index:"0"`
	GenesisValidatorsRoot         []byte                       `json:"genesis_validators_root" ssz-index:"1" ssz-size:"32"`
	Slot                          uint64                       `json:"slot" ssz-index:"2"`
	Fork                          *Fork                        `json:"fork" ssz-index:"3"`
	LatestBlockHeader             *BeaconBlockHeader           `json:"latest_block_header" ssz-index:"4"`
	BlockRoots                    [][]byte                     `json:"block_roots" ssz-index:"5" ssz-size:"8192,32"`
	StateRoots                    [][]byte                     `json:"state_roots" ssz-index:"6" ssz-size:"8192,32"`
	HistoricalRoots               [][]byte                     `json:"historical_roots" ssz-index:"7" ssz-max:"16777216" ssz-size:"?,32"`
	Eth1Data                      *Eth1Data                    `json:"eth1_data" ssz-index:"8"`
	Eth1DataVotes                 []*Eth1Data                  `json:"eth1_data_votes" ssz-index:"9" ssz-max:"2048"`
	Eth1DepositIndex              uint64                       `json:"eth1_deposit_index" ssz-index:"10"`
	Validators                    []*Validator                 `json:"validators" ssz-index:"11" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	Balances                      []uint64                     `json:"balances" ssz-index:"12" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	RandaoMixes                   [][]byte                     `json:"randao_mixes" ssz-index:"13" ssz-size:"65536,32"`
	Slashings                     []uint64                     `json:"slashings" ssz-index:"14" ssz-size:"8192"`
	PreviousEpochParticipation    []byte                       `json:"previous_epoch_participation" ssz-index:"15" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	CurrentEpochParticipation     []byte                       `json:"current_epoch_participation" ssz-index:"16" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	JustificationBits             []byte                       `json:"justification_bits" ssz-index:"17" ssz-size:"1"`
	PreviousJustifiedCheckpoint   *Checkpoint                  `json:"previous_justified_checkpoint" ssz-index:"18"`
	CurrentJustifiedCheckpoint    *Checkpoint                  `json:"current_justified_checkpoint" ssz-index:"19"`
	FinalizedCheckpoint           *Checkpoint                  `json:"finalized_checkpoint" ssz-index:"20"`
	InactivityScores              []uint64                     `json:"inactivity_scores" ssz-index:"21" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	CurrentSyncCommittee          *SyncCommittee               `json:"current_sync_committee" ssz-index:"22"`
	NextSyncCommittee             *SyncCommittee               `json:"next_sync_committee" ssz-index:"23"`
	LatestBlockHash               []byte                       `json:"latest_block_hash" ssz-index:"24" ssz-size:"32"`
	NextWithdrawalIndex           uint64                       `json:"next_withdrawal_index" ssz-index:"25"`
	NextWithdrawalValidatorIndex  uint64                       `json:"next_withdrawal_validator_index" ssz-index:"26"`
	HistoricalSummaries           []*HistoricalSummary         `json:"historical_summaries" ssz-index:"27" ssz-max:"16777216"`
	DepositRequestsStartIndex     uint64                       `json:"deposit_requests_start_index" ssz-index:"28"`
	DepositBalanceToConsume       uint64                       `json:"deposit_balance_to_consume" ssz-index:"29"`
	ExitBalanceToConsume          uint64                       `json:"exit_balance_to_consume" ssz-index:"30"`
	EarliestExitEpoch             uint64                       `json:"earliest_exit_epoch" ssz-index:"31"`
	ConsolidationBalanceToConsume uint64                       `json:"consolidation_balance_to_consume" ssz-index:"32"`
	EarliestConsolidationEpoch    uint64                       `json:"earliest_consolidation_epoch" ssz-index:"33"`
	PendingDeposits               []*PendingDeposit            `json:"pending_deposits" ssz-index:"34" ssz-type:"progressive-list" ssz-max:"134217728"`
	PendingPartialWithdrawals     []*PendingPartialWithdrawal  `json:"pending_partial_withdrawals" ssz-index:"35" ssz-type:"progressive-list" ssz-max:"134217728"`
	PendingConsolidations         []*PendingConsolidation      `json:"pending_consolidations" ssz-index:"36" ssz-type:"progressive-list" ssz-max:"262144"`
	ProposerLookahead             []uint64                     `json:"proposer_lookahead" ssz-index:"37" ssz-size:"64"`
	Builders                      []*Builder                   `json:"builders" ssz-index:"38" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	NextWithdrawalBuilderIndex    uint64                       `json:"next_withdrawal_builder_index" ssz-index:"39"`
	ExecutionPayloadAvailability  []byte                       `json:"execution_payload_availability" ssz-index:"40" ssz-size:"1024"` // Bitvector[8192]
	BuilderPendingPayments        []*BuilderPendingPayment     `json:"builder_pending_payments" ssz-index:"41" ssz-size:"64"`
	BuilderPendingWithdrawals     []*BuilderPendingWithdrawal  `json:"builder_pending_withdrawals" ssz-index:"42" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	LatestExecutionPayloadBid     *ExecutionPayloadBid         `json:"latest_execution_payload_bid" ssz-index:"43"`
	PayloadExpectedWithdrawals    []*Withdrawal                `json:"payload_expected_withdrawals" ssz-index:"44" ssz-type:"progressive-list" ssz-max:"1099511627776"`
	PtcWindow                     [][]uint64                   `json:"ptc_window" ssz-index:"45" ssz-size:"96,512"`
}

type Builder struct {
	Pubkey            []byte `json:"pubkey" ssz-size:"48"`
	Version           uint8  `json:"version"`
	ExecutionAddress  []byte `json:"execution_address" ssz-size:"20"`
	Balance           uint64 `json:"balance"`
	DepositEpoch      uint64 `json:"deposit_epoch"`
	WithdrawableEpoch uint64 `json:"withdrawable_epoch"`
}

type BuilderPendingWithdrawal struct {
	FeeRecipient []byte `json:"fee_recipient" ssz-size:"20"`
	Amount       uint64 `json:"amount"`
	BuilderIndex uint64 `json:"builder_index"`
}

type BuilderPendingPayment struct {
	Weight        uint64                    `json:"weight"`
	Withdrawal    *BuilderPendingWithdrawal `json:"withdrawal"`
	ProposerIndex uint64                    `json:"proposer_index"`
}

// ExecutionPayloadBid is a ProgressiveContainer (EIP-7495). ssz-index tags drive
// dynamic-ssz merkleization; BlobKzgCommitments is a ProgressiveList (EIP-7916).
type ExecutionPayloadBid struct {
	ParentBlockHash       []byte   `json:"parent_block_hash" ssz-index:"0" ssz-size:"32"`
	ParentBlockRoot       []byte   `json:"parent_block_root" ssz-index:"1" ssz-size:"32"`
	BlockHash             []byte   `json:"block_hash" ssz-index:"2" ssz-size:"32"`
	PrevRandao            []byte   `json:"prev_randao" ssz-index:"3" ssz-size:"32"`
	FeeRecipient          []byte   `json:"fee_recipient" ssz-index:"4" ssz-size:"20"`
	GasLimit              uint64   `json:"gas_limit" ssz-index:"5"`
	BuilderIndex          uint64   `json:"builder_index" ssz-index:"6"`
	Slot                  uint64   `json:"slot" ssz-index:"7"`
	Value                 uint64   `json:"value" ssz-index:"8"`
	ExecutionPayment      uint64   `json:"execution_payment" ssz-index:"9"`
	BlobKzgCommitments    [][]byte `json:"blob_kzg_commitments" ssz-index:"10" ssz-type:"progressive-list" ssz-size:"?,48" ssz-max:"4096"`
	ExecutionRequestsRoot []byte   `json:"execution_requests_root" ssz-index:"11" ssz-size:"32"`
}
