// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
)

func TestParseGetLogsBlockRange(t *testing.T) {
	if got := ParseGetLogsBlockRange(""); got != 10 {
		t.Fatalf("empty env should default to 10, got %d", got)
	}
	if got := ParseGetLogsBlockRange("0"); got != 10 {
		t.Fatalf("zero should default to 10, got %d", got)
	}
	if got := ParseGetLogsBlockRange("nope"); got != 10 {
		t.Fatalf("invalid should default to 10, got %d", got)
	}
	if got := ParseGetLogsBlockRange("4096"); got != 4096 {
		t.Fatalf("paid override 4096, got %d", got)
	}
}

func TestParseGetLogsPageDelay(t *testing.T) {
	for raw, want := range map[string]time.Duration{
		"": 0, "nope": 0, "-5": 0, "0": 0, "250": 250 * time.Millisecond,
	} {
		if got := ParseGetLogsPageDelay(raw); got != want {
			t.Fatalf("ParseGetLogsPageDelay(%q) = %v, want %v", raw, got, want)
		}
	}
}

func TestWalkLogFilterPages_PacesWindows(t *testing.T) {
	t.Setenv(GetLogsPageDelayEnv, "20")
	pages := 0
	start := time.Now()
	err := WalkLogFilterPages(context.Background(), 29, func(opts *bind.FilterOpts) (bool, error) {
		pages++
		return false, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	// 3 windows ([20,29], [10,19], [0,9]) need 2 pauses.
	if pages != 3 || time.Since(start) < 40*time.Millisecond {
		t.Fatalf("pages=%d elapsed=%v, want 3 pages paced by 2x20ms", pages, time.Since(start))
	}
}

func TestWalkLogFilterPages_PauseStopsOnCancel(t *testing.T) {
	t.Setenv(GetLogsPageDelayEnv, "60000")
	ctx, cancel := context.WithCancel(context.Background())
	err := WalkLogFilterPages(ctx, 29, func(opts *bind.FilterOpts) (bool, error) {
		cancel()
		return false, nil
	})
	if err != context.Canceled {
		t.Fatalf("want context.Canceled, got %v", err)
	}
}

func TestLogFilterStart_AlchemyFreeTenBlocksInclusive(t *testing.T) {
	// Alchemy's example [0x18b8c13, 0x18b8c1c] is 10 blocks inclusive (to-from+1=10).
	end := uint64(0x18b8c1c)
	start := LogFilterStart(end, 10)
	if start != 0x18b8c13 {
		t.Fatalf("start=%#x want %#x (inclusive 10)", start, uint64(0x18b8c13))
	}
	if end-start+1 != 10 {
		t.Fatalf("inclusive count=%d want 10", end-start+1)
	}
	// Old BlocksPerQuery subtraction (end-10) is 11 blocks and 400s on Free.
	if LogFilterStart(100, 10) != 91 {
		t.Fatalf("end=100 range=10 start=%d want 91", LogFilterStart(100, 10))
	}
	if LogFilterStart(9, 10) != 0 {
		t.Fatalf("near genesis should clamp to 0")
	}
	if LogFilterStart(0, 10) != 0 {
		t.Fatalf("block 0 should stay 0")
	}
}

func TestWalkLogFilterPages_StopsAtDoneAndDoesNotOverlap(t *testing.T) {
	t.Setenv(GetLogsBlockRangeEnv, "10")
	var pages [][2]uint64
	err := WalkLogFilterPages(context.Background(), 100, func(opts *bind.FilterOpts) (bool, error) {
		end := *opts.End
		pages = append(pages, [2]uint64{opts.Start, end})
		if end-opts.Start+1 > 10 {
			t.Fatalf("page %d-%d is %d blocks, Alchemy Free max is 10", opts.Start, end, end-opts.Start+1)
		}
		return opts.Start <= 80, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(pages) < 2 {
		t.Fatalf("expected multiple pages, got %v", pages)
	}
	if pages[0] != [2]uint64{91, 100} {
		t.Fatalf("first page=%v want [91,100]", pages[0])
	}
	if pages[1] != [2]uint64{81, 90} {
		t.Fatalf("second page=%v want [81,90] (no overlap)", pages[1])
	}
}

func TestWalkLogFilterPages_ReachesGenesis(t *testing.T) {
	t.Setenv(GetLogsBlockRangeEnv, "10")
	var last bind.FilterOpts
	n := 0
	err := WalkLogFilterPages(context.Background(), 15, func(opts *bind.FilterOpts) (bool, error) {
		n++
		last = *opts
		return false, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if last.Start != 0 {
		t.Fatalf("last page should include genesis, start=%d", last.Start)
	}
	if n < 2 {
		t.Fatalf("expected more than one page from block 15, got %d", n)
	}
}

func TestNonceBlock(t *testing.T) {
	// Nonce i is emitted at emitted[i-1]; two nonces share block 120.
	emitted := []uint64{100, 120, 120, 5000, 26264405}
	nonceAt := func(_ context.Context, block uint64) (uint64, error) {
		var n uint64
		for _, b := range emitted {
			if b <= block {
				n++
			}
		}
		return n, nil
	}
	const to = 26276435
	for i, want := range emitted {
		got, err := NonceBlock(context.Background(), 0, to, uint64(i+1), nonceAt)
		if err != nil || got != want {
			t.Fatalf("nonce %d: got %d err %v, want %d", i+1, got, err, want)
		}
	}
	// A from hint at the previous nonce's block still finds a shared block.
	if got, _ := NonceBlock(context.Background(), 120, to, 3, nonceAt); got != 120 {
		t.Fatalf("nonce 3 from 120: got %d", got)
	}
	if _, err := NonceBlock(context.Background(), 0, to, 6, nonceAt); err == nil {
		t.Fatal("nonce past the latest block must error")
	}
}

func TestNonceBlockCallCount(t *testing.T) {
	// Mainnet case: the pending nonce is 12,030 blocks behind the tip.
	const emittedAt, to = 26264405, 26276435
	calls := 0
	nonceAt := func(_ context.Context, block uint64) (uint64, error) {
		calls++
		if block >= emittedAt {
			return 1, nil
		}
		return 0, nil
	}
	got, err := NonceBlock(context.Background(), 0, to, 1, nonceAt)
	if err != nil || got != emittedAt {
		t.Fatalf("got %d err %v", got, err)
	}
	if calls > 30 {
		t.Fatalf("%d calls, want <= 30", calls)
	}
}
