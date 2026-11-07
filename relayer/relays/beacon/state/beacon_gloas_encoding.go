package state

// Classic nested-container helpers plus Gloas BeaconState / bid HashTreeRoot via dynamic-ssz.

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"

	ssz "github.com/ferranbt/fastssz"
	"github.com/pk910/dynamic-ssz/treeproof"
)

const (
	slotsPerHistoricalRoot    = 8192
	epochsPerHistoricalVector = 65536
	epochsPerSlashingsVector  = 8192
	proposerLookaheadSize     = 64
	slotsPerEpoch             = 32
	builderPendingPaymentsLen = 2 * slotsPerEpoch       // Vector[BuilderPendingPayment, 2*SLOTS_PER_EPOCH]
	ptcWindowLen              = (1 + 2) * slotsPerEpoch // MIN_SEED_LOOKAHEAD + 2 epochs
	ptcSize                   = 512                     // Vector[ValidatorIndex, PTC_SIZE]
)

func u64Root(v uint64) [32]byte {
	var out [32]byte
	binary.LittleEndian.PutUint64(out[:8], v)
	return out
}

func bytesRoot(b []byte) [32]byte {
	var out [32]byte
	copy(out[:], b)
	return out
}

func boolRoot(b bool) [32]byte {
	var out [32]byte
	if b {
		out[0] = 1
	}
	return out
}

func nextPowerOfTwo(n int) int {
	if n <= 1 {
		return 1
	}
	p := 1
	for p < n {
		p <<= 1
	}
	return p
}

func merkleize(chunks [][32]byte, limit int) [32]byte {
	padded := make([][32]byte, limit)
	copy(padded, chunks)
	for len(padded) > 1 {
		next := make([][32]byte, len(padded)/2)
		for i := range next {
			var buf [64]byte
			copy(buf[:32], padded[2*i][:])
			copy(buf[32:], padded[2*i+1][:])
			next[i] = sha256.Sum256(buf[:])
		}
		padded = next
	}
	if len(padded) == 0 {
		return [32]byte{}
	}
	return padded[0]
}

func container(fields ...[32]byte) [32]byte {
	return merkleize(fields, nextPowerOfTwo(len(fields)))
}

func fixedBytesRoot(data []byte) [32]byte {
	nChunks := (len(data) + 31) / 32
	if nChunks == 0 {
		return [32]byte{}
	}
	chunks := make([][32]byte, nChunks)
	for i := 0; i < nChunks; i++ {
		start := i * 32
		end := start + 32
		if end > len(data) {
			end = len(data)
		}
		copy(chunks[i][:], data[start:end])
	}
	return merkleize(chunks, nextPowerOfTwo(nChunks))
}

func (f *Fork) hashTreeRoot() [32]byte {
	return container(bytesRoot(f.PreviousVersion), bytesRoot(f.CurrentVersion), u64Root(f.Epoch))
}

func (h *BeaconBlockHeader) hashTreeRoot() [32]byte {
	return container(u64Root(h.Slot), u64Root(h.ProposerIndex), bytesRoot(h.ParentRoot),
		bytesRoot(h.StateRoot), bytesRoot(h.BodyRoot))
}

func (c *Checkpoint) hashTreeRoot() [32]byte {
	return container(u64Root(c.Epoch), bytesRoot(c.Root))
}

// FieldRoots returns the checkpoint's two field roots (epoch, root).
func (c *Checkpoint) FieldRoots() [][32]byte {
	return [][32]byte{u64Root(c.Epoch), bytesRoot(c.Root)}
}

func (e *Eth1Data) hashTreeRoot() [32]byte {
	return container(bytesRoot(e.DepositRoot), u64Root(e.DepositCount), bytesRoot(e.BlockHash))
}

func (h *HistoricalSummary) hashTreeRoot() [32]byte {
	return container(bytesRoot(h.BlockSummaryRoot), bytesRoot(h.StateSummaryRoot))
}

func (v *Validator) hashTreeRoot() [32]byte {
	return container(fixedBytesRoot(v.Pubkey), bytesRoot(v.WithdrawalCredentials),
		u64Root(v.EffectiveBalance), boolRoot(v.Slashed), u64Root(v.ActivationEligibilityEpoch),
		u64Root(v.ActivationEpoch), u64Root(v.ExitEpoch), u64Root(v.WithdrawableEpoch))
}

func (s *SyncCommittee) hashTreeRoot() [32]byte {
	keys := make([][32]byte, len(s.PubKeys))
	for i, k := range s.PubKeys {
		keys[i] = fixedBytesRoot(k)
	}
	return container(merkleize(keys, nextPowerOfTwo(len(keys))), fixedBytesRoot(s.AggregatePubKey[:]))
}

func (b *Builder) hashTreeRoot() [32]byte {
	return container(fixedBytesRoot(b.Pubkey), u64Root(uint64(b.Version)),
		bytesRoot(b.ExecutionAddress), u64Root(b.Balance), u64Root(b.DepositEpoch),
		u64Root(b.WithdrawableEpoch))
}

func (w *BuilderPendingWithdrawal) hashTreeRoot() [32]byte {
	return container(bytesRoot(w.FeeRecipient), u64Root(w.Amount), u64Root(w.BuilderIndex))
}

func (p *BuilderPendingPayment) hashTreeRoot() [32]byte {
	return container(u64Root(p.Weight), p.Withdrawal.hashTreeRoot(), u64Root(p.ProposerIndex))
}

func (w *Withdrawal) hashTreeRoot() [32]byte {
	return container(u64Root(w.Index), u64Root(w.ValidatorIndex), bytesRoot(w.Address[:]),
		u64Root(w.Amount))
}

// bidSSZ is ExecutionPayloadBid without methods so dynamic-ssz does not recurse
// through hashTreeRoot when building nested roots.
type bidSSZ ExecutionPayloadBid

func (b *ExecutionPayloadBid) hashTreeRoot() ([32]byte, error) {
	return gloasSSZ.HashTreeRoot((*bidSSZ)(b))
}

// stateSSZ is BeaconStateGloas without methods so dynamic-ssz does not recurse
// through HashTreeRoot / GetTree.
type stateSSZ BeaconStateGloas

// HashTreeRoot computes the Gloas BeaconState root via dynamic-ssz progressive merkleization.
func (s *BeaconStateGloas) HashTreeRoot() ([32]byte, error) {
	return gloasSSZ.HashTreeRoot((*stateSSZ)(s))
}

// GloasProofTree returns the dynamic-ssz tree used by the beacon-state prover.
func (s *BeaconStateGloas) GloasProofTree() (*treeproof.Node, error) {
	return gloasSSZ.GetTree((*stateSSZ)(s))
}

func (s *BeaconStateGloas) GetSlot() uint64 { return s.Slot }

func (s *BeaconStateGloas) GetLatestBlockHeader() *BeaconBlockHeader { return s.LatestBlockHeader }

func (s *BeaconStateGloas) GetBlockRoots() [][]byte { return s.BlockRoots }

func (s *BeaconStateGloas) GetFinalizedCheckpoint() *Checkpoint { return s.FinalizedCheckpoint }

func (s *BeaconStateGloas) GetCurrentSyncCommittee() *SyncCommittee { return s.CurrentSyncCommittee }

func (s *BeaconStateGloas) GetNextSyncCommittee() *SyncCommittee { return s.NextSyncCommittee }

// GetTree is deliberately unsupported for fastssz Node. Use GloasProofTree for progressive proofs.
func (s *BeaconStateGloas) GetTree() (*ssz.Node, error) {
	return nil, fmt.Errorf("GetTree is not supported for Gloas: use GloasProofTree")
}
