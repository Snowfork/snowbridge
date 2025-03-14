package scale

import (
	"errors"
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

type BeaconCheckpoint struct {
	Header                     BeaconHeader
	CurrentSyncCommittee       SyncCommittee
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
	BlockRootsRoot             types.H256
	BlockRootsBranch           []types.H256
}

type Update struct {
	Payload                  UpdatePayload
	FinalizedHeaderBlockRoot common.Hash
	BlockRootsTree           *ssz.Node
}

type ProofPayload struct {
	HeaderPayload    HeaderUpdatePayload
	FinalizedPayload *Update
}

type UpdatePayload struct {
	AttestedHeader          BeaconHeader
	SyncAggregate           SyncAggregate
	SignatureSlot           types.U64
	NextSyncCommitteeUpdate OptionNextSyncCommitteeUpdatePayload
	FinalizedHeader         BeaconHeader
	FinalityBranch          []types.H256
	BlockRootsRoot          types.H256
	BlockRootsBranch        []types.H256
}
type OptionNextSyncCommitteeUpdatePayload struct {
	HasValue bool
	Value    NextSyncCommitteeUpdatePayload
}

type NextSyncCommitteeUpdatePayload struct {
	NextSyncCommittee       SyncCommittee
	NextSyncCommitteeBranch []types.H256
}

func (o OptionNextSyncCommitteeUpdatePayload) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.HasValue, o.Value)
}

func (o *OptionNextSyncCommitteeUpdatePayload) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.HasValue, &o.Value)
}

type HeaderUpdatePayload struct {
	Header          BeaconHeader
	AncestryProof   OptionAncestryProof
	ExecutionHeader VersionedExecutionPayloadHeader
	ExecutionBranch []types.H256
}

type OptionAncestryProof struct {
	HasValue bool
	Value    AncestryProof
}

type AncestryProof struct {
	HeaderBranch       []types.H256
	FinalizedBlockRoot types.H256
}

func (o OptionAncestryProof) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.HasValue, o.Value)
}

func (o *OptionAncestryProof) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.HasValue, &o.Value)
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

type CompactBeaconState struct {
	Slot           types.UCompact
	BlockRootsRoot types.H256
}

type BeaconState struct {
	CompactBeaconState
	BlockRoot types.H256
}

type VersionedExecutionPayloadHeader struct {
	Deneb *ExecutionPayloadHeaderDeneb
}

var (
	ErrEncodeVersionedExecutionPayloadHeader = errors.New("error scale encode VersionedExecutionPayloadHeader")
)

func (v VersionedExecutionPayloadHeader) Encode(encoder scale.Encoder) error {
	var err error
	if v.Deneb != nil {
		encoder.PushByte(1)
		err = encoder.Encode(v.Deneb)
	} else {
		err = ErrEncodeVersionedExecutionPayloadHeader
	}
	return err
}

type CompactExecutionHeader struct {
	ParentHash   types.H256
	BlockNumber  types.UCompact
	StateRoot    types.H256
	ReceiptsRoot types.H256
}
