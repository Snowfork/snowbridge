package beefy

import (
	"math/big"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

func TestIsHandoverRequest(t *testing.T) {
	state := &BeefyClientState{NextValidatorSetID: 5900}
	mk := func(leafNext uint64) *Request {
		var task Request
		task.Proof.Leaf.BeefyNextAuthoritySet.ID = types.U64(leafNext)
		return &task
	}
	if isHandoverRequest(state, mk(5900)) {
		t.Fatal("same next set must not be handover")
	}
	if !isHandoverRequest(state, mk(5901)) {
		t.Fatal("higher leaf next set must be handover")
	}
	if isHandoverRequest(state, nil) || isHandoverRequest(nil, mk(5901)) {
		t.Fatal("nil inputs must not be handover")
	}
}

func TestTipGasAction(t *testing.T) {
	cases := []struct {
		path       beefyGasPath
		wantSubmit bool
		wantTwoPh  bool
		wantErr    bool
	}{
		{pathFiatShamir, true, false, false},
		{pathForceTwoPhase, true, true, false},
		{pathTwoPhase, false, false, false},
		{pathWait, false, false, false},
		{beefyGasPath(99), false, false, true},
	}
	for _, tc := range cases {
		submit, twoPhase, err := tipGasAction(tc.path)
		if tc.wantErr {
			if err == nil {
				t.Fatalf("path=%s: want error", tc.path)
			}
			continue
		}
		if err != nil {
			t.Fatalf("path=%s: unexpected err %v", tc.path, err)
		}
		if submit != tc.wantSubmit || twoPhase != tc.wantTwoPh {
			t.Fatalf("path=%s: got submit=%v twoPhase=%v want %v %v",
				tc.path, submit, twoPhase, tc.wantSubmit, tc.wantTwoPh)
		}
	}
}

func TestTipLagAllows(t *testing.T) {
	orig := BeefyTipMinLagBlocks
	BeefyTipMinLagBlocks = 2400
	t.Cleanup(func() { BeefyTipMinLagBlocks = orig })

	if tipLagAllows(2399) {
		t.Fatal("lag below tip min must refuse")
	}
	if !tipLagAllows(2400) {
		t.Fatal("lag at tip min must allow")
	}
	if !tipLagAllows(2401) {
		t.Fatal("lag above tip min must allow")
	}
}

func TestSelectBeefyGasPath(t *testing.T) {
	origFiat, origWait, origLag := BeefyFiatShamirMaxWei, BeefyWaitMinWei, BeefySkipMaxLagBlocks
	BeefyFiatShamirMaxWei = 100_000_000
	BeefyWaitMinWei = 250_000_000
	BeefySkipMaxLagBlocks = 100
	t.Cleanup(func() {
		BeefyFiatShamirMaxWei = origFiat
		BeefyWaitMinWei = origWait
		BeefySkipMaxLagBlocks = origLag
	})

	cases := []struct {
		name string
		gas  uint64
		lag  uint64
		want beefyGasPath
	}{
		{"cheap", 50_000_000, 10, pathFiatShamir},
		{"mid", 200_000_000, 10, pathTwoPhase},
		{"expensive_within_window", 300_000_000, 50, pathWait},
		{"expensive_past_window", 300_000_000, 101, pathForceTwoPhase},
		{"boundary_fiat", 99_999_999, 1, pathFiatShamir},
		{"boundary_mid", 100_000_000, 1, pathTwoPhase},
		{"boundary_wait", 250_000_000, 100, pathWait},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := selectBeefyGasPath(new(big.Int).SetUint64(tc.gas), tc.lag)
			if got != tc.want {
				t.Fatalf("gas=%d lag=%d: got %s want %s", tc.gas, tc.lag, got, tc.want)
			}
		})
	}
}
