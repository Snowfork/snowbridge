package state

import ssz "github.com/ferranbt/fastssz"

type DepositRequestsContainer struct {
	DepositRequests []*DepositRequest `ssz-max:"8192" json:"deposit_requests"`
}

type WithdrawalRequestsContainer struct {
	WithdrawalRequests []*WithdrawalRequest `ssz-max:"16" json:"withdrawal_requests"`
}

type ConsolidationRequestsContainer struct {
	ConsolidationRequests []*ConsolidationRequest `ssz-max:"1" json:"consolidation_requests"`
}

type DepositRequest struct {
	Pubkey                [48]byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials [32]byte `ssz-size:"32" json:"withdrawal_credentials"`
	Amount                uint64   `json:"amount"`
	Signature             [96]byte `json:"signature,omitempty" ssz-size:"96"`
	Index                 uint64   `json:"index,omitempty"`
}

type PendingDeposit struct {
	Pubkey                [48]byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials [32]byte `ssz-size:"32" json:"withdrawal_credentials"`
	Amount                uint64   `json:"amount"`
	Signature             [96]byte `json:"signature,omitempty" ssz-size:"96"`
	Index                 uint64   `json:"index,omitempty"`
}

type WithdrawalRequest struct {
	SourceAddress   [20]byte `ssz-size:"20" json:"source_address" `
	ValidatorPubkey [48]byte `ssz-size:"48" json:"validator_pubkey"`
	Amount          uint64   `json:"amount"`
}

type ConsolidationRequest struct {
	SourceAddress [20]byte `ssz-size:"20" json:"source_address" `
	SourcePubkey  [48]byte `ssz-size:"48" json:"source_pubkey"`
	TargetPubkey  [48]byte `ssz-size:"48" json:"target_pubkey"`
}

type SignedBeaconBlockElectra struct {
	Message   BeaconBlockElectra
	Signature [96]byte `json:"signature,omitempty" ssz-size:"96"`
}

type BeaconBlockElectra struct {
	Slot          uint64                  `json:"slot"`
	ProposerIndex uint64                  `json:"proposer_index"`
	ParentRoot    []byte                  `json:"parent_root" ssz-size:"32"`
	StateRoot     []byte                  `json:"state_root" ssz-size:"32"`
	Body          *BeaconBlockBodyElectra `json:"body"`
}

type BeaconBlockBodyElectra struct {
	RandaoReveal          []byte                        `json:"randao_reveal" ssz-size:"96"`
	Eth1Data              *Eth1Data                     `json:"eth1_data"`
	Graffiti              [32]byte                      `json:"graffiti" ssz-size:"32"`
	ProposerSlashings     []*ProposerSlashing           `json:"proposer_slashings" ssz-max:"16"`
	AttesterSlashings     []*AttesterSlashingElectra    `json:"attester_slashings" ssz-max:"1"` // Modified in Electra
	Attestations          []*AttestationElectra         `json:"attestations" ssz-max:"8"`       // Modified in Electra
	Deposits              []*Deposit                    `json:"deposits" ssz-max:"16"`
	VoluntaryExits        []*SignedVoluntaryExit        `json:"voluntary_exits" ssz-max:"16"`
	SyncAggregate         *SyncAggregateMainnet         `json:"sync_aggregate"`
	ExecutionPayload      *ExecutionPayloadDeneb        `json:"execution_payload"`
	BlsToExecutionChanges []*SignedBLSToExecutionChange `json:"bls_to_execution_changes,omitempty" ssz-max:"16"`
	BlobKzgCommitments    [][48]byte                    `json:"blob_kzg_commitments,omitempty" ssz-max:"4096" ssz-size:"?,48"`
	ExecutionRequests     *ExecutionRequests            `json:"execution_requests"` // New in Electra
}

type BeaconStateElectra struct {
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
	DepositRequestsStartIndex     uint64                       `json:"deposit_requests_start_index,omitempty"`                    // New in Electra
	DepositBalanceToConsume       uint64                       `json:"deposit_balance_to_consume,omitempty"`                      // New in Electra
	ExitBalanceToConsume          uint64                       `json:"exit_balance_to_consume,omitempty"`                         // New in Electra
	EarliestExitEpoch             uint64                       `json:"earliest_exit_epoch,omitempty"`                             // New in Electra
	ConsolidationBalanceToConsume uint64                       `json:"consolidation_balance_to_consume,omitempty"`                // New in Electra
	EarliestConsolidationEpoch    uint64                       `json:"earliest_consolidation_epoch,omitempty"`                    // New in Electra
	PendingDeposits               []*PendingDeposit            `json:"pending_deposits,omitempty" ssz-max:"134217728"`            // New in Electra
	PendingPartialWithdrawals     []*PendingPartialWithdrawal  `json:"pending_partial_withdrawals,omitempty" ssz-max:"134217728"` // New in Electra
	PendingConsolidations         []*PendingConsolidation      `json:"pending_consolidations,omitempty" ssz-max:"262144"`         // New in Electra
}

type AttestationElectra struct {
	AggregationBits []byte           `json:"aggregation_bits" ssz:"bitlist" ssz-max:"131072"` // Modified in Electra
	Data            *AttestationData `json:"data"`
	Signature       [96]byte         `json:"signature" ssz-size:"96"`
	CommitteeBits   []byte           `json:"committee_bits" cast-type:"github.com/prysmaticlabs/go-bitfield.Bitvector64" ssz-size:"8"` // New in Electra
}

type AttesterSlashingElectra struct {
	Attestation1 *IndexedAttestationElectra `json:"attestation_1"`
	Attestation2 *IndexedAttestationElectra `json:"attestation_2"`
}

type IndexedAttestationElectra struct {
	AttestationIndices []uint64         `json:"attesting_indices" ssz-max:"131072"` // Modified in Electra
	Data               *AttestationData `json:"data"`
	Signature          [96]byte         `json:"signature" ssz-size:"96"`
}

type PendingPartialWithdrawal struct {
	Index             uint64 `json:"index"`
	Amount            uint64 `json:"amount"`
	WithdrawableEpoch uint64 `json:"withdrawable_epoch"`
}

type PendingConsolidation struct {
	SourceIndex uint64 `json:"source_index"`
	TargetIndex uint64 `json:"target_index"`
}

type ExecutionRequests struct {
	Deposits       []*DepositRequest       `json:"deposit_requests,omitempty" ssz-max:"8192"`     // New in Electra
	Withdrawals    []*WithdrawalRequest    `json:"withdrawals_requests,omitempty" ssz-max:"16"`   // New in Electra
	Consolidations []*ConsolidationRequest `json:"consolidations_requests,omitempty" ssz-max:"2"` // New in Electra
}

func (b *BeaconBlockElectra) GetBeaconSlot() uint64 {
	return b.Slot
}

func (b *BeaconBlockElectra) GetBlockBodyTree() (*ssz.Node, error) {
	return b.Body.GetTree()
}

func (b *BeaconBlockElectra) ExecutionPayloadDeneb() *ExecutionPayloadDeneb {
	return b.Body.ExecutionPayload
}

func (b *BeaconStateElectra) GetSlot() uint64 {
	return b.Slot
}

func (b *BeaconStateElectra) GetLatestBlockHeader() *BeaconBlockHeader {
	return b.LatestBlockHeader
}

func (b *BeaconStateElectra) GetBlockRoots() [][]byte {
	return b.BlockRoots
}

func (b *BeaconStateElectra) SetBlockRoots(blockRoots [][]byte) {
	b.BlockRoots = blockRoots
}

func (b *BeaconStateElectra) GetFinalizedCheckpoint() *Checkpoint {
	return b.FinalizedCheckpoint
}

func (b *BeaconStateElectra) GetNextSyncCommittee() *SyncCommittee {
	return b.NextSyncCommittee
}
func (b *BeaconStateElectra) GetCurrentSyncCommittee() *SyncCommittee {
	return b.CurrentSyncCommittee
}

func (b *SignedBeaconBlockElectra) GetBlock() BeaconBlock {
	return &b.Message
}
