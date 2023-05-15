package scale

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type BlockRootProof struct {
	Leaf  types.H256
	Proof []types.H256
	Tree  *ssz.Node
}

type CheckPoint struct {
	Header                     BeaconHeader
	CurrentSyncCommittee       SyncCommittee
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
	ImportTime                 types.U64
}

type SyncCommitteePeriodUpdate struct {
	Payload                  SyncCommitteePeriodPayload
	FinalizedHeaderBlockRoot common.Hash
	BlockRootsTree           *ssz.Node
}

type SyncCommitteePeriodPayload struct {
	AttestedHeader          BeaconHeader
	NextSyncCommittee       SyncCommittee
	NextSyncCommitteeBranch []types.H256
	FinalizedHeader         BeaconHeader
	FinalityBranch          []types.H256
	SyncAggregate           SyncAggregate
	SyncCommitteePeriod     types.U64
	SignatureSlot           types.U64
	BlockRootsHash          types.H256
	BlockRootProof          []types.H256
}

type FinalizedHeaderPayload struct {
	AttestedHeader  BeaconHeader
	FinalizedHeader BeaconHeader
	FinalityBranch  []types.H256
	SyncAggregate   SyncAggregate
	SignatureSlot   types.U64
	BlockRootsHash  types.H256
	BlockRootProof  []types.H256
}

type FinalizedHeaderUpdate struct {
	Payload                  FinalizedHeaderPayload
	FinalizedHeaderBlockRoot common.Hash
	BlockRootsTree           *ssz.Node
}

type HeaderUpdatePayload struct {
	BeaconHeader              BeaconHeader
	ExecutionHeader           ExecutionPayloadHeaderCapella
	ExecutionBranch           []types.H256
	SyncAggregate             SyncAggregate
	SignatureSlot             types.U64
	BlockRootBranch           []types.H256
	BlockRootBranchHeaderRoot types.H256
}

type HeaderUpdate struct {
	Payload           HeaderUpdatePayload
	NextSyncAggregate SyncAggregate
}

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

type SignedVoluntaryExit struct {
	Exit      VoluntaryExit
	Signature []byte
}

type VoluntaryExit struct {
	Epoch          types.U64
	ValidaterIndex types.U64
}

type BLSToExecutionChange struct {
	ValidatorIndex     types.U64
	FromBlsPubkey      []byte
	ToExecutionAddress []byte
}

type SignedBLSToExecutionChange struct {
	Message   *BLSToExecutionChange
	Signature []byte
}

type ExecutionPayloadHeaderCapella struct {
	ParentHash       types.H256
	FeeRecipient     types.H160
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
	WithdrawalsRoot  types.H256
}

type Body struct {
	RandaoReveal      []byte
	Eth1Data          Eth1Data
	Graffiti          types.H256
	ProposerSlashings []ProposerSlashing
	AttesterSlashings []AttesterSlashing
	Attestations      []Attestation
	Deposits          []Deposit
	VoluntaryExits    []SignedVoluntaryExit
	SyncAggregate     SyncAggregate
	ExecutionPayload  ExecutionPayloadHeaderCapella
}

type BodyCapella struct {
	RandaoReveal          []byte
	Eth1Data              Eth1Data
	Graffiti              types.H256
	ProposerSlashings     []ProposerSlashing
	AttesterSlashings     []AttesterSlashing
	Attestations          []Attestation
	Deposits              []Deposit
	VoluntaryExits        []SignedVoluntaryExit
	SyncAggregate         SyncAggregate
	ExecutionPayload      ExecutionPayloadHeaderCapella
	BlsToExecutionChanges []*SignedBLSToExecutionChange
}

type BeaconBlock struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	Body          Body
}

type BeaconBlockCapella struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	Body          BodyCapella
}

type SyncCommittee struct {
	Pubkeys         [][48]byte
	AggregatePubkey [48]byte
}

// Use a custom SCALE encoder to encode SyncCommitteeBits as fixed array
func (s SyncCommittee) Encode(encoder scale.Encoder) error {

	switch len(s.Pubkeys) {
	case 32:
		var pubkeys [32][48]byte
		copy(pubkeys[:], s.Pubkeys)
		encoder.Encode(pubkeys)
	case 512:
		var pubkeys [512][48]byte
		copy(pubkeys[:], s.Pubkeys)
		encoder.Encode(pubkeys)
	default:
		return fmt.Errorf("invalid sync committee size")
	}
	encoder.Encode(s.AggregatePubkey)
	return nil
}

type SyncAggregate struct {
	SyncCommitteeBits      []byte
	SyncCommitteeSignature [96]byte
}

// Use a custom SCALE encoder to encode SyncCommitteeBits as fixed array
func (s SyncAggregate) Encode(encoder scale.Encoder) error {

	switch len(s.SyncCommitteeBits) {
	case 4:
		//	32 / 8 = 4
		var syncCommitteeBits [4]byte
		copy(syncCommitteeBits[:], s.SyncCommitteeBits)
		encoder.Encode(syncCommitteeBits)
	case 64:
		//	512 / 8 = 64
		var syncCommitteeBits [64]byte
		copy(syncCommitteeBits[:], s.SyncCommitteeBits)
		encoder.Encode(syncCommitteeBits)
	default:
		return fmt.Errorf("invalid sync committee size")
	}
	encoder.Encode(s.SyncCommitteeSignature)
	return nil
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
