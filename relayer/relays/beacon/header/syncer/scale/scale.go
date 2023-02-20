package scale

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type BeaconHeader struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	BodyRoot      types.H256
}

type Eth1Data struct {
	DepositRoot  types.H256
	DepositCount types.U64
	BlockHash    types.H256
}

type SignedHeader struct {
	Message   BeaconHeader
	Signature []byte
}

type Checkpoint struct {
	Epoch types.U64
	Root  types.H256
}

type ProposerSlashing struct {
	SignedHeader1 SignedHeader
	SignedHeader2 SignedHeader
}

type AttestationData struct {
	Slot            types.U64
	Index           types.U64
	BeaconBlockRoot types.H256
	Source          Checkpoint
	Target          Checkpoint
}

type IndexedAttestation struct {
	AttestingIndices []types.U64
	Data             AttestationData
	Signature        []byte
}

type Attestation struct {
	AggregationBits []byte
	Data            AttestationData
	Signature       []byte
}

type AttesterSlashing struct {
	Attestation1 IndexedAttestation
	Attestation2 IndexedAttestation
}

type DepositData struct {
	Pubkey                []byte
	WithdrawalCredentials types.H256
	Amount                types.U64
	Signature             []byte
}

type Deposit struct {
	Proof []types.H256
	Data  DepositData
}

type DepositVoluntaryExit struct {
	Proof []types.H256
	Data  DepositData
}

type VoluntaryExit struct {
	Epoch          types.U64
	ValidaterIndex types.U64
}

type ExecutionPayload struct {
	ParentHash       types.H256
	FeeRecipient     []byte
	StateRoot        types.H256
	ReceiptsRoot     types.H256
	LogsBloom        []byte
	PrevRandao       types.H256
	BlockNumber      types.U64
	GasLimit         types.U64
	GasUsed          types.U64
	Timestamp        types.U64
	ExtraData        []byte
	BaseFeePerGas    types.U256
	BlockHash        types.H256
	TransactionsRoot types.H256
}

type Body struct {
	RandaoReveal      []byte
	Eth1Data          Eth1Data
	Graffiti          types.H256
	ProposerSlashings []ProposerSlashing
	AttesterSlashings []AttesterSlashing
	Attestations      []Attestation
	Deposits          []Deposit
	VoluntaryExits    []VoluntaryExit
	SyncAggregate     SyncAggregate
	ExecutionPayload  ExecutionPayload
}

type BeaconBlock struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	Body          Body
}

type CurrentSyncCommittee struct {
	Pubkeys         [][48]byte
	AggregatePubkey [48]byte
}

type SyncAggregate struct {
	SyncCommitteeBits      []byte
	SyncCommitteeSignature []byte
}

func (b *BeaconHeader) ToSSZ() *state.BeaconBlockHeader {
	return &state.BeaconBlockHeader{
		Slot:          uint64(b.Slot),
		ProposerIndex: uint64(b.ProposerIndex),
		ParentRoot:    common.FromHex(b.ParentRoot.Hex()),
		StateRoot:     common.FromHex(b.StateRoot.Hex()),
		BodyRoot:      common.FromHex(b.BodyRoot.Hex()),
	}
}
