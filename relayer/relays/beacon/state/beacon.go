package state

import (
	ssz "github.com/ferranbt/fastssz"
)

type Checkpoint struct {
	Epoch uint64 `json:"epoch"`
	Root  []byte `json:"root" ssz-size:"32"`
}

type Slot uint64 // alias from the same package

type Hash [32]byte

type AttestationData struct {
	Slot            Slot        `json:"slot"`
	Index           uint64      `json:"index"`
	BeaconBlockHash Hash        `json:"beacon_block_root" ssz-size:"32"`
	Source          *Checkpoint `json:"source"`
	Target          *Checkpoint `json:"target"`
}

type Attestation struct {
	AggregationBits []byte           `json:"aggregation_bits" ssz:"bitlist" ssz-max:"2048"`
	Data            *AttestationData `json:"data"`
	Signature       [96]byte         `json:"signature" ssz-size:"96"`
}

type DepositData struct {
	Pubkey                [48]byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials [32]byte `json:"withdrawal_credentials" ssz-size:"32"`
	Amount                uint64   `json:"amount"`
	Signature             []byte   `json:"signature" ssz-size:"96"`
	Root                  [32]byte `ssz:"-"`
}

type Deposit struct {
	Proof [][]byte `ssz-size:"33,32"`
	Data  *DepositData
}

type IndexedAttestation struct {
	AttestationIndices []uint64         `json:"attesting_indices" ssz-max:"2048"`
	Data               *AttestationData `json:"data"`
	Signature          []byte           `json:"signature" ssz-size:"96"`
}

type Fork struct {
	PreviousVersion []byte `json:"previous_version" ssz-size:"4"`
	CurrentVersion  []byte `json:"current_version" ssz-size:"4"`
	Epoch           uint64 `json:"epoch"`
}

type Validator struct {
	Pubkey                     []byte `json:"pubkey" ssz-size:"48"`
	WithdrawalCredentials      []byte `json:"withdrawal_credentials" ssz-size:"32"`
	EffectiveBalance           uint64 `json:"effective_balance"`
	Slashed                    bool   `json:"slashed"`
	ActivationEligibilityEpoch uint64 `json:"activation_eligibility_epoch"`
	ActivationEpoch            uint64 `json:"activation_epoch"`
	ExitEpoch                  uint64 `json:"exit_epoch"`
	WithdrawableEpoch          uint64 `json:"withdrawable_epoch"`
}

type VoluntaryExit struct {
	Epoch          uint64 `json:"epoch"`
	ValidatorIndex uint64 `json:"validator_index"`
}

type SignedVoluntaryExit struct {
	Exit      *VoluntaryExit `json:"message"`
	Signature [96]byte       `json:"signature" ssz-size:"96"`
}

type Eth1Data struct {
	DepositRoot  []byte `json:"deposit_root" ssz-size:"32"`
	DepositCount uint64 `json:"deposit_count"`
	BlockHash    []byte `json:"block_hash" ssz-size:"32"`
}

type ProposerSlashing struct {
	Header1 *SignedBeaconBlockHeader `json:"signed_header_1"`
	Header2 *SignedBeaconBlockHeader `json:"signed_header_2"`
}

type AttesterSlashing struct {
	Attestation1 *IndexedAttestation `json:"attestation_1"`
	Attestation2 *IndexedAttestation `json:"attestation_2"`
}

type BlockRootsContainerMainnet struct {
	BlockRoots [][]byte `json:"block_roots" ssz-size:"8192,32"`
}

type TransactionsRootContainer struct {
	Transactions [][]byte `ssz-max:"1048576,1073741824" ssz-size:"?,?" json:"transactions"`
}

type SignedBeaconBlockHeader struct {
	Header    *BeaconBlockHeader `json:"message"`
	Signature []byte             `json:"signature" ssz-size:"96"`
}

type BeaconBlockHeader struct {
	Slot          uint64 `json:"slot"`
	ProposerIndex uint64 `json:"proposer_index"`
	ParentRoot    []byte `json:"parent_root" ssz-size:"32"`
	StateRoot     []byte `json:"state_root" ssz-size:"32"`
	BodyRoot      []byte `json:"body_root" ssz-size:"32"`
}

type SyncCommittee struct {
	PubKeys         [][]byte `json:"pubkeys" ssz-size:"512,48"`
	AggregatePubKey [48]byte `json:"aggregate_pubkey" ssz-size:"48"`
}

type SyncAggregateMainnet struct {
	SyncCommitteeBits      []byte   `json:"sync_committee_bits" ssz-size:"64"`
	SyncCommitteeSignature [96]byte `json:"sync_committee_signature" ssz-size:"96"`
}

type BeaconState interface {
	UnmarshalSSZ(buf []byte) error
	GetSlot() uint64
	GetLatestBlockHeader() *BeaconBlockHeader
	GetBlockRoots() [][]byte
	GetTree() (*ssz.Node, error)
	GetFinalizedCheckpoint() *Checkpoint
	GetCurrentSyncCommittee() *SyncCommittee
	GetNextSyncCommittee() *SyncCommittee
}

type SyncAggregate interface {
	GetSyncAggregateBits() []byte
	GetSyncAggregateSignature() [96]byte
}

type BeaconBlockBody interface {
	GetTree() (*ssz.Node, error)
}

type BlockRootsContainer interface {
	GetTree() (*ssz.Node, error)
	SetBlockRoots(blockRoots [][]byte)
}

type BeaconBlock interface {
	UnmarshalSSZ(buf []byte) error
	GetBeaconSlot() uint64
	ExecutionPayloadDeneb() *ExecutionPayloadDeneb
	GetTree() (*ssz.Node, error)
	GetBlockBodyTree() (*ssz.Node, error)
}

type SignedBeaconBlock interface {
	UnmarshalSSZ(buf []byte) error
	GetBlock() BeaconBlock
}

func (b *BlockRootsContainerMainnet) SetBlockRoots(blockRoots [][]byte) {
	b.BlockRoots = blockRoots
}

func (s *SyncAggregateMainnet) GetSyncAggregateBits() []byte {
	return s.SyncCommitteeBits
}

func (s *SyncAggregateMainnet) GetSyncAggregateSignature() [96]byte {
	return s.SyncCommitteeSignature
}

type BLSToExecutionChange struct {
	ValidatorIndex     uint64 `json:"validator_index,omitempty"`
	FromBlsPubkey      []byte `json:"from_bls_pubkey,omitempty" ssz-size:"48"`
	ToExecutionAddress []byte `json:"to_execution_address,omitempty" ssz-size:"20"`
}

type SignedBLSToExecutionChange struct {
	Message   *BLSToExecutionChange `json:"message,omitempty"`
	Signature []byte                `json:"signature,omitempty" ssz-size:"96"`
}

type HistoricalSummary struct {
	BlockSummaryRoot []byte `json:"block_summary_root,omitempty" ssz-size:"32"`
	StateSummaryRoot []byte `json:"state_summary_root,omitempty" ssz-size:"32"`
}

type Withdrawal struct {
	Index          uint64   `json:"index"`
	ValidatorIndex uint64   `json:"validator_index"`
	Address        [20]byte `json:"address" ssz-size:"20"`
	Amount         uint64   `json:"amount"`
}

type WithdrawalsRootContainerMainnet struct {
	Withdrawals []*Withdrawal `ssz-max:"16" json:"withdrawals"`
}
