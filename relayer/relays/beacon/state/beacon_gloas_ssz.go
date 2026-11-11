package state

// Hand-written SSZ decoding for the Gloas BeaconState.
//
// EIP-7688 changes merkleization only: "ProgressiveContainer / ProgressiveList serialize in
// the same way as Container / List". So the wire format here is ordinary SSZ -- a fixed part
// holding fixed-size fields and 4-byte offsets, followed by the variable part in field order.
// It is hand-written rather than generated because the generator that would produce it also
// produces a wrong HashTreeRoot, and keeping the two halves together makes that trap obvious.

import (
	"encoding/binary"
	"fmt"
)

const (
	sizePendingDeposit           = 192
	sizePendingPartialWithdrawal = 24
	sizePendingConsolidation     = 16
	sizeValidator                = 121
	sizeHistoricalSummary        = 64
	sizeEth1Data                 = 72
	sizeBuilder                  = 93
	sizeBuilderPendingWithdrawal = 36
	sizeBuilderPendingPayment    = 52
	sizeWithdrawal               = 44
	sizeCheckpoint               = 40
	sizeSyncCommittee            = 512*48 + 48
	sizeAvailabilityBits         = slotsPerHistoricalRoot / 8
)

// reader walks a buffer, refusing to read past the end.
type reader struct {
	b   []byte
	pos int
}

func (r *reader) take(n int) ([]byte, error) {
	if n < 0 || r.pos+n > len(r.b) {
		return nil, fmt.Errorf("ssz: want %d bytes at %d, have %d", n, r.pos, len(r.b))
	}
	out := r.b[r.pos : r.pos+n]
	r.pos += n
	return out, nil
}

func (r *reader) u64() (uint64, error) {
	b, err := r.take(8)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint64(b), nil
}

func (r *reader) offset() (int, error) {
	b, err := r.take(4)
	if err != nil {
		return 0, err
	}
	return int(binary.LittleEndian.Uint32(b)), nil
}

func (r *reader) bytes(n int) ([]byte, error) {
	b, err := r.take(n)
	if err != nil {
		return nil, err
	}
	out := make([]byte, n)
	copy(out, b)
	return out, nil
}

// roots reads n consecutive 32-byte roots.
func (r *reader) roots(n int) ([][]byte, error) {
	out := make([][]byte, n)
	for i := range out {
		b, err := r.bytes(32)
		if err != nil {
			return nil, err
		}
		out[i] = b
	}
	return out, nil
}

func (r *reader) u64s(n int) ([]uint64, error) {
	out := make([]uint64, n)
	for i := range out {
		v, err := r.u64()
		if err != nil {
			return nil, err
		}
		out[i] = v
	}
	return out, nil
}

// fixedList splits a variable-part slice into equal-sized elements.
func fixedList(seg []byte, size int, what string) (int, error) {
	if size == 0 || len(seg)%size != 0 {
		return 0, fmt.Errorf("ssz: %s segment %d is not a multiple of %d", what, len(seg), size)
	}
	return len(seg) / size, nil
}

// UnmarshalSSZ decodes a Gloas BeaconState.
func (s *BeaconStateGloas) UnmarshalSSZ(buf []byte) error {
	r := &reader{b: buf}
	var err error
	// Offsets into the variable part, in field order.
	var offHistoricalRoots, offEth1DataVotes, offValidators, offBalances int
	var offPrevParticipation, offCurrParticipation, offInactivity, offSummaries int
	var offPendingDeposits, offPendingPartials, offPendingConsolidations int
	var offBuilders, offBuilderWithdrawals, offBid, offExpectedWithdrawals int

	if s.GenesisTime, err = r.u64(); err != nil {
		return err
	}
	if s.GenesisValidatorsRoot, err = r.bytes(32); err != nil {
		return err
	}
	if s.Slot, err = r.u64(); err != nil {
		return err
	}
	if s.Fork, err = readFork(r); err != nil {
		return err
	}
	if s.LatestBlockHeader, err = readBlockHeader(r); err != nil {
		return err
	}
	if s.BlockRoots, err = r.roots(slotsPerHistoricalRoot); err != nil {
		return err
	}
	if s.StateRoots, err = r.roots(slotsPerHistoricalRoot); err != nil {
		return err
	}
	if offHistoricalRoots, err = r.offset(); err != nil {
		return err
	}
	if s.Eth1Data, err = readEth1Data(r); err != nil {
		return err
	}
	if offEth1DataVotes, err = r.offset(); err != nil {
		return err
	}
	if s.Eth1DepositIndex, err = r.u64(); err != nil {
		return err
	}
	if offValidators, err = r.offset(); err != nil {
		return err
	}
	if offBalances, err = r.offset(); err != nil {
		return err
	}
	if s.RandaoMixes, err = r.roots(epochsPerHistoricalVector); err != nil {
		return err
	}
	if s.Slashings, err = r.u64s(epochsPerSlashingsVector); err != nil {
		return err
	}
	if offPrevParticipation, err = r.offset(); err != nil {
		return err
	}
	if offCurrParticipation, err = r.offset(); err != nil {
		return err
	}
	if s.JustificationBits, err = r.bytes(1); err != nil {
		return err
	}
	if s.PreviousJustifiedCheckpoint, err = readCheckpoint(r); err != nil {
		return err
	}
	if s.CurrentJustifiedCheckpoint, err = readCheckpoint(r); err != nil {
		return err
	}
	if s.FinalizedCheckpoint, err = readCheckpoint(r); err != nil {
		return err
	}
	if offInactivity, err = r.offset(); err != nil {
		return err
	}
	if s.CurrentSyncCommittee, err = readSyncCommittee(r); err != nil {
		return err
	}
	if s.NextSyncCommittee, err = readSyncCommittee(r); err != nil {
		return err
	}
	if s.LatestBlockHash, err = r.bytes(32); err != nil {
		return err
	}
	if s.NextWithdrawalIndex, err = r.u64(); err != nil {
		return err
	}
	if s.NextWithdrawalValidatorIndex, err = r.u64(); err != nil {
		return err
	}
	if offSummaries, err = r.offset(); err != nil {
		return err
	}
	if s.DepositRequestsStartIndex, err = r.u64(); err != nil {
		return err
	}
	if s.DepositBalanceToConsume, err = r.u64(); err != nil {
		return err
	}
	if s.ExitBalanceToConsume, err = r.u64(); err != nil {
		return err
	}
	if s.EarliestExitEpoch, err = r.u64(); err != nil {
		return err
	}
	if s.ConsolidationBalanceToConsume, err = r.u64(); err != nil {
		return err
	}
	if s.EarliestConsolidationEpoch, err = r.u64(); err != nil {
		return err
	}
	if offPendingDeposits, err = r.offset(); err != nil {
		return err
	}
	if offPendingPartials, err = r.offset(); err != nil {
		return err
	}
	if offPendingConsolidations, err = r.offset(); err != nil {
		return err
	}
	if s.ProposerLookahead, err = r.u64s(proposerLookaheadSize); err != nil {
		return err
	}
	if offBuilders, err = r.offset(); err != nil {
		return err
	}
	if s.NextWithdrawalBuilderIndex, err = r.u64(); err != nil {
		return err
	}
	if s.ExecutionPayloadAvailability, err = r.bytes(sizeAvailabilityBits); err != nil {
		return err
	}
	s.BuilderPendingPayments = make([]*BuilderPendingPayment, builderPendingPaymentsLen)
	for i := range s.BuilderPendingPayments {
		if s.BuilderPendingPayments[i], err = readBuilderPendingPayment(r); err != nil {
			return err
		}
	}
	if offBuilderWithdrawals, err = r.offset(); err != nil {
		return err
	}
	if offBid, err = r.offset(); err != nil {
		return err
	}
	if offExpectedWithdrawals, err = r.offset(); err != nil {
		return err
	}
	s.PtcWindow = make([][]uint64, ptcWindowLen)
	for i := range s.PtcWindow {
		if s.PtcWindow[i], err = r.u64s(ptcSize); err != nil {
			return err
		}
	}
	return s.unmarshalVariable(buf, []int{
		offHistoricalRoots, offEth1DataVotes, offValidators, offBalances,
		offPrevParticipation, offCurrParticipation, offInactivity, offSummaries,
		offPendingDeposits, offPendingPartials, offPendingConsolidations,
		offBuilders, offBuilderWithdrawals, offBid, offExpectedWithdrawals,
	})
}

func readFork(r *reader) (*Fork, error) {
	prev, err := r.bytes(4)
	if err != nil {
		return nil, err
	}
	curr, err := r.bytes(4)
	if err != nil {
		return nil, err
	}
	epoch, err := r.u64()
	if err != nil {
		return nil, err
	}
	return &Fork{PreviousVersion: prev, CurrentVersion: curr, Epoch: epoch}, nil
}

func readBlockHeader(r *reader) (*BeaconBlockHeader, error) {
	slot, err := r.u64()
	if err != nil {
		return nil, err
	}
	proposer, err := r.u64()
	if err != nil {
		return nil, err
	}
	parent, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	state, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	body, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	return &BeaconBlockHeader{Slot: slot, ProposerIndex: proposer, ParentRoot: parent, StateRoot: state, BodyRoot: body}, nil
}

func readEth1Data(r *reader) (*Eth1Data, error) {
	root, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	count, err := r.u64()
	if err != nil {
		return nil, err
	}
	hash, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	return &Eth1Data{DepositRoot: root, DepositCount: count, BlockHash: hash}, nil
}

func readCheckpoint(r *reader) (*Checkpoint, error) {
	epoch, err := r.u64()
	if err != nil {
		return nil, err
	}
	root, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	return &Checkpoint{Epoch: epoch, Root: root}, nil
}

func readSyncCommittee(r *reader) (*SyncCommittee, error) {
	keys := make([][]byte, 512)
	for i := range keys {
		k, err := r.bytes(48)
		if err != nil {
			return nil, err
		}
		keys[i] = k
	}
	agg, err := r.bytes(48)
	if err != nil {
		return nil, err
	}
	var a [48]byte
	copy(a[:], agg)
	return &SyncCommittee{PubKeys: keys, AggregatePubKey: a}, nil
}

func readBuilderPendingWithdrawal(r *reader) (*BuilderPendingWithdrawal, error) {
	fee, err := r.bytes(20)
	if err != nil {
		return nil, err
	}
	amount, err := r.u64()
	if err != nil {
		return nil, err
	}
	idx, err := r.u64()
	if err != nil {
		return nil, err
	}
	return &BuilderPendingWithdrawal{FeeRecipient: fee, Amount: amount, BuilderIndex: idx}, nil
}

func readBuilderPendingPayment(r *reader) (*BuilderPendingPayment, error) {
	weight, err := r.u64()
	if err != nil {
		return nil, err
	}
	w, err := readBuilderPendingWithdrawal(r)
	if err != nil {
		return nil, err
	}
	proposer, err := r.u64()
	if err != nil {
		return nil, err
	}
	return &BuilderPendingPayment{Weight: weight, Withdrawal: w, ProposerIndex: proposer}, nil
}

func readBuilder(r *reader) (*Builder, error) {
	pk, err := r.bytes(48)
	if err != nil {
		return nil, err
	}
	ver, err := r.take(1)
	if err != nil {
		return nil, err
	}
	addr, err := r.bytes(20)
	if err != nil {
		return nil, err
	}
	bal, err := r.u64()
	if err != nil {
		return nil, err
	}
	dep, err := r.u64()
	if err != nil {
		return nil, err
	}
	wd, err := r.u64()
	if err != nil {
		return nil, err
	}
	return &Builder{Pubkey: pk, Version: ver[0], ExecutionAddress: addr, Balance: bal, DepositEpoch: dep, WithdrawableEpoch: wd}, nil
}

func readValidator(r *reader) (*Validator, error) {
	pk, err := r.bytes(48)
	if err != nil {
		return nil, err
	}
	cred, err := r.bytes(32)
	if err != nil {
		return nil, err
	}
	eff, err := r.u64()
	if err != nil {
		return nil, err
	}
	sl, err := r.take(1)
	if err != nil {
		return nil, err
	}
	elig, err := r.u64()
	if err != nil {
		return nil, err
	}
	act, err := r.u64()
	if err != nil {
		return nil, err
	}
	exit, err := r.u64()
	if err != nil {
		return nil, err
	}
	wd, err := r.u64()
	if err != nil {
		return nil, err
	}
	return &Validator{Pubkey: pk, WithdrawalCredentials: cred, EffectiveBalance: eff, Slashed: sl[0] == 1,
		ActivationEligibilityEpoch: elig, ActivationEpoch: act, ExitEpoch: exit, WithdrawableEpoch: wd}, nil
}

func readWithdrawal(r *reader) (*Withdrawal, error) {
	idx, err := r.u64()
	if err != nil {
		return nil, err
	}
	vidx, err := r.u64()
	if err != nil {
		return nil, err
	}
	addr, err := r.bytes(20)
	if err != nil {
		return nil, err
	}
	amount, err := r.u64()
	if err != nil {
		return nil, err
	}
	var a [20]byte
	copy(a[:], addr)
	return &Withdrawal{Index: idx, ValidatorIndex: vidx, Address: a, Amount: amount}, nil
}

// unmarshalVariable decodes the variable part. Each field runs from its own offset to the
// next one, and the last runs to the end of the buffer. Offsets must be non-decreasing and
// within the buffer, or the state is malformed.
func (s *BeaconStateGloas) unmarshalVariable(buf []byte, offs []int) error {
	seg := func(i int) ([]byte, error) {
		start := offs[i]
		end := len(buf)
		if i+1 < len(offs) {
			end = offs[i+1]
		}
		if start > end || end > len(buf) {
			return nil, fmt.Errorf("ssz: bad offset pair %d..%d in %d bytes", start, end, len(buf))
		}
		return buf[start:end], nil
	}
	// Decodes a segment of equal-sized elements with a per-element reader.
	list := func(i, size int, what string, put func(*reader, int) error) error {
		b, err := seg(i)
		if err != nil {
			return err
		}
		n, err := fixedList(b, size, what)
		if err != nil {
			return err
		}
		r := &reader{b: b}
		return put(r, n)
	}

	b, err := seg(0) // historical_roots
	if err != nil {
		return err
	}
	n, err := fixedList(b, 32, "historical_roots")
	if err != nil {
		return err
	}
	if s.HistoricalRoots, err = (&reader{b: b}).roots(n); err != nil {
		return err
	}

	if err := list(1, sizeEth1Data, "eth1_data_votes", func(r *reader, n int) error {
		s.Eth1DataVotes = make([]*Eth1Data, n)
		for i := range s.Eth1DataVotes {
			if s.Eth1DataVotes[i], err = readEth1Data(r); err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(2, sizeValidator, "validators", func(r *reader, n int) error {
		s.Validators = make([]*Validator, n)
		for i := range s.Validators {
			if s.Validators[i], err = readValidator(r); err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(3, 8, "balances", func(r *reader, n int) error {
		s.Balances, err = r.u64s(n)
		return err
	}); err != nil {
		return err
	}

	if s.PreviousEpochParticipation, err = seg(4); err != nil {
		return err
	}
	if s.CurrentEpochParticipation, err = seg(5); err != nil {
		return err
	}

	if err := list(6, 8, "inactivity_scores", func(r *reader, n int) error {
		s.InactivityScores, err = r.u64s(n)
		return err
	}); err != nil {
		return err
	}

	if err := list(7, sizeHistoricalSummary, "historical_summaries", func(r *reader, n int) error {
		s.HistoricalSummaries = make([]*HistoricalSummary, n)
		for i := range s.HistoricalSummaries {
			a, e := r.bytes(32)
			if e != nil {
				return e
			}
			c, e := r.bytes(32)
			if e != nil {
				return e
			}
			s.HistoricalSummaries[i] = &HistoricalSummary{BlockSummaryRoot: a, StateSummaryRoot: c}
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(8, sizePendingDeposit, "pending_deposits", func(r *reader, n int) error {
		s.PendingDeposits = make([]*PendingDeposit, n)
		for i := range s.PendingDeposits {
			d := &PendingDeposit{}
			pk, e := r.bytes(48)
			if e != nil {
				return e
			}
			copy(d.Pubkey[:], pk)
			cr, e := r.bytes(32)
			if e != nil {
				return e
			}
			copy(d.WithdrawalCredentials[:], cr)
			if d.Amount, e = r.u64(); e != nil {
				return e
			}
			sig, e := r.bytes(96)
			if e != nil {
				return e
			}
			copy(d.Signature[:], sig)
			if d.Index, e = r.u64(); e != nil {
				return e
			}
			s.PendingDeposits[i] = d
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(9, sizePendingPartialWithdrawal, "pending_partial_withdrawals", func(r *reader, n int) error {
		s.PendingPartialWithdrawals = make([]*PendingPartialWithdrawal, n)
		for i := range s.PendingPartialWithdrawals {
			w := &PendingPartialWithdrawal{}
			var e error
			if w.Index, e = r.u64(); e != nil {
				return e
			}
			if w.Amount, e = r.u64(); e != nil {
				return e
			}
			if w.WithdrawableEpoch, e = r.u64(); e != nil {
				return e
			}
			s.PendingPartialWithdrawals[i] = w
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(10, sizePendingConsolidation, "pending_consolidations", func(r *reader, n int) error {
		s.PendingConsolidations = make([]*PendingConsolidation, n)
		for i := range s.PendingConsolidations {
			c := &PendingConsolidation{}
			var e error
			if c.SourceIndex, e = r.u64(); e != nil {
				return e
			}
			if c.TargetIndex, e = r.u64(); e != nil {
				return e
			}
			s.PendingConsolidations[i] = c
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(11, sizeBuilder, "builders", func(r *reader, n int) error {
		s.Builders = make([]*Builder, n)
		for i := range s.Builders {
			if s.Builders[i], err = readBuilder(r); err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		return err
	}

	if err := list(12, sizeBuilderPendingWithdrawal, "builder_pending_withdrawals", func(r *reader, n int) error {
		s.BuilderPendingWithdrawals = make([]*BuilderPendingWithdrawal, n)
		for i := range s.BuilderPendingWithdrawals {
			if s.BuilderPendingWithdrawals[i], err = readBuilderPendingWithdrawal(r); err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		return err
	}

	bidSeg, err := seg(13)
	if err != nil {
		return err
	}
	if s.LatestExecutionPayloadBid, err = readExecutionPayloadBid(bidSeg); err != nil {
		return err
	}

	if err := list(14, sizeWithdrawal, "payload_expected_withdrawals", func(r *reader, n int) error {
		s.PayloadExpectedWithdrawals = make([]*Withdrawal, n)
		for i := range s.PayloadExpectedWithdrawals {
			if s.PayloadExpectedWithdrawals[i], err = readWithdrawal(r); err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		return err
	}
	return nil
}

// readExecutionPayloadBid decodes the bid, whose only variable field is blob_kzg_commitments.
func readExecutionPayloadBid(buf []byte) (*ExecutionPayloadBid, error) {
	r := &reader{b: buf}
	b := &ExecutionPayloadBid{}
	var err error
	if b.ParentBlockHash, err = r.bytes(32); err != nil {
		return nil, err
	}
	if b.ParentBlockRoot, err = r.bytes(32); err != nil {
		return nil, err
	}
	if b.BlockHash, err = r.bytes(32); err != nil {
		return nil, err
	}
	if b.PrevRandao, err = r.bytes(32); err != nil {
		return nil, err
	}
	if b.FeeRecipient, err = r.bytes(20); err != nil {
		return nil, err
	}
	if b.GasLimit, err = r.u64(); err != nil {
		return nil, err
	}
	if b.BuilderIndex, err = r.u64(); err != nil {
		return nil, err
	}
	if b.Slot, err = r.u64(); err != nil {
		return nil, err
	}
	if b.Value, err = r.u64(); err != nil {
		return nil, err
	}
	if b.ExecutionPayment, err = r.u64(); err != nil {
		return nil, err
	}
	off, err := r.offset()
	if err != nil {
		return nil, err
	}
	if b.ExecutionRequestsRoot, err = r.bytes(32); err != nil {
		return nil, err
	}
	if off > len(buf) {
		return nil, fmt.Errorf("ssz: bid commitments offset %d beyond %d", off, len(buf))
	}
	seg := buf[off:]
	n, err := fixedList(seg, 48, "blob_kzg_commitments")
	if err != nil {
		return nil, err
	}
	cr := &reader{b: seg}
	b.BlobKzgCommitments = make([][]byte, n)
	for i := range b.BlobKzgCommitments {
		if b.BlobKzgCommitments[i], err = cr.bytes(48); err != nil {
			return nil, err
		}
	}
	return b, nil
}
