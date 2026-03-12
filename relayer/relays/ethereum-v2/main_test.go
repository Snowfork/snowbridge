package execution

import (
	"errors"
	"fmt"
	"slices"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockRelay wraps Relay and allows mocking fetchUnprocessedParachainNonces
type mockRelay struct {
	*Relay
	mockFetchUnprocessedParachainNonces func(latest uint64) ([]uint64, error)
}

// Override fetchUnprocessedParachainNonces to use the mock
func (m *mockRelay) fetchUnprocessedParachainNonces(latest uint64) ([]uint64, error) {
	if m.mockFetchUnprocessedParachainNonces != nil {
		return m.mockFetchUnprocessedParachainNonces(latest)
	}
	return m.Relay.fetchUnprocessedParachainNonces(latest)
}

// isMessageProcessed is a copy of the method for testing with mockRelay
func (m *mockRelay) isMessageProcessed(eventNonce uint64) (bool, error) {
	paraNonces, err := m.fetchUnprocessedParachainNonces(eventNonce)
	if err != nil {
		return false, fmt.Errorf("fetch latest parachain nonce: %w", err)
	}

	if slices.Contains(paraNonces, eventNonce) {
		return false, nil
	}

	return true, nil
}

func TestIsMessageProcessed(t *testing.T) {
	tests := []struct {
		name                  string
		eventNonce            uint64
		unprocessedNonces     []uint64
		fetchError            error
		expectedProcessed     bool
		expectedError         bool
		expectedErrorContains string
	}{
		{
			name:              "message not processed - nonce in unprocessed list",
			eventNonce:        5,
			unprocessedNonces: []uint64{3, 5, 7},
			fetchError:        nil,
			expectedProcessed: false,
			expectedError:     false,
		},
		{
			name:              "message processed - nonce not in unprocessed list",
			eventNonce:        10,
			unprocessedNonces: []uint64{3, 5, 7},
			fetchError:        nil,
			expectedProcessed: true,
			expectedError:     false,
		},
		{
			name:              "message processed - empty unprocessed list",
			eventNonce:        1,
			unprocessedNonces: []uint64{},
			fetchError:        nil,
			expectedProcessed: true,
			expectedError:     false,
		},
		{
			name:                  "error fetching nonces",
			eventNonce:            5,
			unprocessedNonces:     nil,
			fetchError:            errors.New("connection error"),
			expectedProcessed:     false,
			expectedError:         true,
			expectedErrorContains: "fetch latest parachain nonce",
		},
		{
			name:              "message not processed - nonce at beginning of list",
			eventNonce:        1,
			unprocessedNonces: []uint64{1, 2, 3},
			fetchError:        nil,
			expectedProcessed: false,
			expectedError:     false,
		},
		{
			name:              "message not processed - nonce at end of list",
			eventNonce:        100,
			unprocessedNonces: []uint64{50, 75, 100},
			fetchError:        nil,
			expectedProcessed: false,
			expectedError:     false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create a mock relay
			relay := &mockRelay{
				Relay: &Relay{},
			}

			// Mock the fetchUnprocessedParachainNonces method
			relay.mockFetchUnprocessedParachainNonces = func(latest uint64) ([]uint64, error) {
				assert.Equal(t, tt.eventNonce, latest, "fetchUnprocessedParachainNonces should be called with eventNonce")
				if tt.fetchError != nil {
					return nil, tt.fetchError
				}
				return tt.unprocessedNonces, nil
			}

			// Call the method under test
			processed, err := relay.isMessageProcessed(tt.eventNonce)

			// Verify results
			if tt.expectedError {
				require.Error(t, err)
				if tt.expectedErrorContains != "" {
					assert.Contains(t, err.Error(), tt.expectedErrorContains)
				}
			} else {
				require.NoError(t, err)
				assert.Equal(t, tt.expectedProcessed, processed)
			}
		})
	}
}
