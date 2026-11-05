package state

// Merkleization of the operation lists carried in a Gloas BeaconBlockBody via dynamic-ssz.

import (
	"crypto/sha256"
)

// AttestationDataGloas is a classic container.
type AttestationDataGloas struct {
	Slot            uint64
	Index           uint64
	BeaconBlockRoot []byte `ssz-size:"32"`
	Source          *Checkpoint
	Target          *Checkpoint
}

// AttestationGloas is a ProgressiveContainer of four active fields.
type AttestationGloas struct {
	AggregationBits []byte                `ssz-index:"0" ssz-type:"progressive-bitlist" ssz-max:"131072"`
	Data            *AttestationDataGloas `ssz-index:"1"`
	Signature       []byte                `ssz-index:"2" ssz-size:"96"`
	CommitteeBits   []byte                `ssz-index:"3" ssz-size:"8"`
}

type attestationSSZ AttestationGloas

func (a *AttestationGloas) hashTreeRoot() ([32]byte, error) {
	return gloasSSZ.HashTreeRoot((*attestationSSZ)(a))
}

type PayloadAttestationDataGloas struct {
	BeaconBlockRoot   []byte `ssz-size:"32"`
	Slot              uint64
	PayloadPresent    bool
	BlobDataAvailable bool
}

// PayloadAttestationGloas is a ProgressiveContainer of three active fields.
type PayloadAttestationGloas struct {
	AggregationBits []byte                       `ssz-index:"0" ssz-size:"64"`
	Data            *PayloadAttestationDataGloas `ssz-index:"1"`
	Signature       []byte                       `ssz-index:"2" ssz-size:"96"`
}

type payloadAttestationSSZ PayloadAttestationGloas

func (p *PayloadAttestationGloas) hashTreeRoot() ([32]byte, error) {
	return gloasSSZ.HashTreeRoot((*payloadAttestationSSZ)(p))
}

type attestationsListSSZ struct {
	Items []*AttestationGloas `ssz-type:"progressive-list" ssz-max:"131072"`
}

type payloadAttestationsListSSZ struct {
	Items []*PayloadAttestationGloas `ssz-type:"progressive-list" ssz-max:"131072"`
}

// AttestationsRoot merkleizes the attestations list.
func AttestationsRoot(list []*AttestationGloas) ([32]byte, error) {
	return gloasSSZ.HashTreeRoot(&attestationsListSSZ{Items: list})
}

// PayloadAttestationsRoot merkleizes the payload attestations list.
func PayloadAttestationsRoot(list []*PayloadAttestationGloas) ([32]byte, error) {
	return gloasSSZ.HashTreeRoot(&payloadAttestationsListSSZ{Items: list})
}

// EmptyProgressiveListRoot is the root of any empty progressive list.
func EmptyProgressiveListRoot() [32]byte {
	root, err := AttestationsRoot(nil)
	if err != nil {
		panic(err)
	}
	return root
}

// ErrUnsupportedBodyList is returned when a block carries an operation the relayer cannot
// merkleize.
type ErrUnsupportedBodyList struct{ Field string }

func (e ErrUnsupportedBodyList) Error() string {
	return "gloas body carries " + e.Field + ", whose merkleization is not implemented"
}

// ExecutionRequestsGloas is a ProgressiveContainer of five lists. Only the all-empty case is
// supported.
type ExecutionRequestsGloas struct {
	Deposits        int
	Withdrawals     int
	Consolidations  int
	BuilderDeposits int
	BuilderExits    int
}

type emptyExecutionRequestsSSZ struct {
	Deposits        []byte `ssz-index:"0" ssz-type:"progressive-list" ssz-max:"1"`
	Withdrawals     []byte `ssz-index:"1" ssz-type:"progressive-list" ssz-max:"1"`
	Consolidations  []byte `ssz-index:"2" ssz-type:"progressive-list" ssz-max:"1"`
	BuilderDeposits []byte `ssz-index:"3" ssz-type:"progressive-list" ssz-max:"1"`
	BuilderExits    []byte `ssz-index:"4" ssz-type:"progressive-list" ssz-max:"1"`
}

func (r *ExecutionRequestsGloas) HashTreeRoot() ([32]byte, error) {
	for field, n := range map[string]int{
		"execution requests.deposits":         r.Deposits,
		"execution requests.withdrawals":      r.Withdrawals,
		"execution requests.consolidations":   r.Consolidations,
		"execution requests.builder_deposits": r.BuilderDeposits,
		"execution requests.builder_exits":    r.BuilderExits,
	} {
		if n > 0 {
			return [32]byte{}, ErrUnsupportedBodyList{Field: field}
		}
	}
	return gloasSSZ.HashTreeRoot(&emptyExecutionRequestsSSZ{})
}

type syncAggregateSSZ struct {
	SyncCommitteeBits      []byte `ssz-size:"64"`
	SyncCommitteeSignature []byte `ssz-size:"96"`
}

// SyncAggregateRoot merkleizes a sync aggregate: classic container of bits + signature.
func SyncAggregateRoot(bits, signature []byte) [32]byte {
	root, err := gloasSSZ.HashTreeRoot(&syncAggregateSSZ{
		SyncCommitteeBits:      bits,
		SyncCommitteeSignature: signature,
	})
	if err != nil {
		return classicContainer(packBytesRoot(bits), packBytesRoot(signature))
	}
	return root
}

func packBytesRoot(data []byte) [32]byte {
	if len(data) <= 32 {
		var out [32]byte
		copy(out[:], data)
		return out
	}
	// Chunk and merkleize classically (BytesRoot for >32).
	nChunks := (len(data) + 31) / 32
	limit := 1
	for limit < nChunks {
		limit <<= 1
	}
	chunks := make([][32]byte, limit)
	for i := 0; i < nChunks; i++ {
		start := i * 32
		end := start + 32
		if end > len(data) {
			end = len(data)
		}
		copy(chunks[i][:], data[start:end])
	}
	return classicContainer(chunks...)
}

func classicContainer(fields ...[32]byte) [32]byte {
	n := 1
	for n < len(fields) {
		n <<= 1
	}
	chunks := make([][32]byte, n)
	copy(chunks, fields)
	for len(chunks) > 1 {
		next := make([][32]byte, len(chunks)/2)
		for i := 0; i < len(next); i++ {
			var buf [64]byte
			copy(buf[:32], chunks[2*i][:])
			copy(buf[32:], chunks[2*i+1][:])
			next[i] = sha256.Sum256(buf[:])
		}
		chunks = next
	}
	if len(chunks) == 0 {
		return [32]byte{}
	}
	return chunks[0]
}
