package executionV3

import (
	"testing"

	"github.com/stretchr/testify/require"

	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
)

func TestV1DestinationSS58EmptyPayload(t *testing.T) {
	r := NewRelay(&Config{}, nil)
	r.config.Sink.SS58Prefix = 0

	// Undecodable empty payload should error (same as legacy getTransactionDestination).
	_, err := r.v1DestinationSS58(&contractsv1.GatewayOutboundMessageAccepted{Payload: nil})
	require.Error(t, err)

	_, err = r.v1DestinationSS58(&contractsv1.GatewayOutboundMessageAccepted{Payload: []byte{}})
	require.Error(t, err)
}
