package ethereum

import (
	"errors"
	"testing"

	"github.com/snowfork/snowbridge/relayer/config"
)

func TestSubmitEndpointTrimmed(t *testing.T) {
	co := NewConnection(&config.EthereumConfig{
		Endpoint:       "https://example.invalid",
		SubmitEndpoint: "  https://rpc.flashbots.net/fast  ",
	}, nil)
	if got := co.SubmitEndpoint(); got != "https://rpc.flashbots.net/fast" {
		t.Fatalf("SubmitEndpoint()=%q", got)
	}
	if NewConnection(&config.EthereumConfig{Endpoint: "https://example.invalid"}, nil).SubmitEndpoint() != "" {
		t.Fatal("empty SubmitEndpoint should yield empty string")
	}
}

func TestIsBenignSubmitError(t *testing.T) {
	if !isBenignSubmitError(errors.New("already known")) {
		t.Fatal("already known should be benign")
	}
	if isBenignSubmitError(errors.New("insufficient funds")) {
		t.Fatal("insufficient funds must not be benign")
	}
}
