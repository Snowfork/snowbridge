package reward

import (
	"errors"
	"fmt"
	"testing"

	geth "github.com/ethereum/go-ethereum"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
)

func TestIsBeaconHistoryUnavailableError(t *testing.T) {
	cases := []struct {
		err  error
		want bool
	}{
		{nil, false},
		{api.ErrNotFound, true},
		{fmt.Errorf("get beacon header by blockRoot: %w", api.ErrNotFound), true},
		{fmt.Errorf("get header update against on-chain finalized: boom"), true},
		{fmt.Errorf("find checkpoint on-chain for slot 1: timeout"), true},
		{errors.New("submit message to outbound queue v2: Invalid Transaction"), false},
		{header.ErrBeaconHeaderNotFinalized, false},
	}
	for _, tc := range cases {
		if got := isBeaconHistoryUnavailableError(tc.err); got != tc.want {
			t.Fatalf("isBeaconHistoryUnavailableError(%v)=%v want %v", tc.err, got, tc.want)
		}
	}
}

func TestIsUnrecoverableBeaconCheckpointError(t *testing.T) {
	if !isUnrecoverableBeaconCheckpointError(fmt.Errorf("find checkpoint on-chain for slot 9: no checkpoint on chain for slot 9")) {
		t.Fatal("expected permanent checkpoint miss")
	}
	if isUnrecoverableBeaconCheckpointError(api.ErrNotFound) {
		t.Fatal("ErrNotFound is not a permanent checkpoint miss")
	}
}

func TestIsBlockNotMinedError(t *testing.T) {
	cases := []struct {
		err  error
		want bool
	}{
		{nil, false},
		{geth.NotFound, true},
		{fmt.Errorf("get block header: %w", geth.NotFound), true},
		{errors.New("not found"), true},
		{errors.New("connection refused"), false},
		{header.ErrBeaconHeaderNotFinalized, false},
	}
	for _, tc := range cases {
		if got := isBlockNotMinedError(tc.err); got != tc.want {
			t.Fatalf("isBlockNotMinedError(%v)=%v want %v", tc.err, got, tc.want)
		}
	}
}
