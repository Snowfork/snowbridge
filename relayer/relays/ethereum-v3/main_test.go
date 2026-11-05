package executionV3

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

func TestIsIgnoredNonce(t *testing.T) {
	r := &Relay{
		config: &Config{
			IgnoreNonces: []uint64{7, 42, 264},
		},
	}

	assert.True(t, r.isIgnoredNonce(264))
	assert.True(t, r.isIgnoredNonce(42))
	assert.False(t, r.isIgnoredNonce(1))
}

func TestTriggerIfFinalizedSlotAdvanced(t *testing.T) {
	r := &Relay{
		trigger: make(chan wakeKind, 1),
	}

	r.triggerIfFinalizedSlotAdvanced(100)
	assert.Equal(t, uint64(100), r.latestFinalizedSlot)
	select {
	case kind := <-r.trigger:
		assert.Equal(t, wakeSSE, kind)
	default:
		t.Fatal("expected worker trigger for new slot")
	}

	r.triggerIfFinalizedSlotAdvanced(100)
	assert.Equal(t, uint64(100), r.latestFinalizedSlot)
	select {
	case <-r.trigger:
		t.Fatal("duplicate slot should not trigger worker again")
	default:
	}

	r.triggerIfFinalizedSlotAdvanced(99)
	select {
	case <-r.trigger:
		t.Fatal("stale slot should not trigger worker")
	default:
	}

	r.triggerIfFinalizedSlotAdvanced(101)
	assert.Equal(t, uint64(101), r.latestFinalizedSlot)
	select {
	case kind := <-r.trigger:
		assert.Equal(t, wakeSSE, kind)
	default:
		t.Fatal("expected worker trigger for advanced slot")
	}
}

func TestTriggerIfOnchainBeaconSlotAdvanced(t *testing.T) {
	r := &Relay{
		trigger:                 make(chan wakeKind, 1),
		latestOnchainBeaconSlot: 100,
	}

	r.triggerIfOnchainBeaconSlotAdvanced(100)
	select {
	case <-r.trigger:
		t.Fatal("duplicate on-chain slot should not trigger")
	default:
	}

	r.triggerIfOnchainBeaconSlotAdvanced(99)
	select {
	case <-r.trigger:
		t.Fatal("stale on-chain slot should not trigger")
	default:
	}

	r.triggerIfOnchainBeaconSlotAdvanced(101)
	assert.Equal(t, uint64(101), r.latestOnchainBeaconSlot)
	select {
	case kind := <-r.trigger:
		assert.Equal(t, wakeOnchainLC, kind)
		assert.False(t, kind.allowBeaconSubmit())
	default:
		t.Fatal("expected worker trigger for advanced on-chain slot")
	}
}

func TestOnchainBeaconTriggerIndependentOfSSESlot(t *testing.T) {
	// SSE already processed eth checkpoint 200; competitor then lands LC at 200.
	// On-chain gate must still wake the worker for message-only submit.
	r := &Relay{
		trigger:                 make(chan wakeKind, 1),
		latestFinalizedSlot:     200,
		latestOnchainBeaconSlot: 192,
	}

	r.triggerIfOnchainBeaconSlotAdvanced(200)
	assert.Equal(t, uint64(200), r.latestOnchainBeaconSlot)
	assert.Equal(t, uint64(200), r.latestFinalizedSlot)
	select {
	case kind := <-r.trigger:
		assert.Equal(t, wakeOnchainLC, kind)
	default:
		t.Fatal("expected on-chain LC advance to trigger even when SSE slot already seen")
	}
}

func TestMergeWake(t *testing.T) {
	assert.Equal(t, wakeOnchainLC, mergeWake(wakeOnchainLC, wakeOnchainLC))
	assert.Equal(t, wakeOnchainLC, mergeWake(wakeOnchainLC, wakeMessagePoll))
	assert.Equal(t, wakeMessagePoll, mergeWake(wakeMessagePoll, wakeMessagePoll))
	assert.Equal(t, wakeSSE, mergeWake(wakeOnchainLC, wakeSSE))
	assert.Equal(t, wakeSSE, mergeWake(wakeSSE, wakeOnchainLC))
	assert.Equal(t, wakeSSE, mergeWake(wakeMessagePoll, wakeSSE))
	assert.Equal(t, wakeStartup, mergeWake(wakeStartup, wakeOnchainLC))
	assert.True(t, wakeSSE.allowBeaconSubmit())
	assert.True(t, wakeStartup.allowBeaconSubmit())
	assert.False(t, wakeOnchainLC.allowBeaconSubmit())
	assert.False(t, wakeMessagePoll.allowBeaconSubmit())
}

func TestFinalityUpdateCoversRequired(t *testing.T) {
	// Lagging LC equal to on-chain (nonce 612 pattern: SSE ahead, finality_update stale).
	assert.False(t, finalityUpdateCoversRequired(14764320, 14764320, 14764352))
	// Ahead of on-chain but not yet at scan/candidate required slot.
	assert.False(t, finalityUpdateCoversRequired(14764351, 14764320, 14764352))
	// Ready: covers scan and advances LC.
	assert.True(t, finalityUpdateCoversRequired(14764352, 14764320, 14764352))
	assert.True(t, finalityUpdateCoversRequired(14764384, 14764320, 14764352))
	// Must be strictly ahead of on-chain even if required equals on-chain.
	assert.False(t, finalityUpdateCoversRequired(100, 100, 100))
	assert.True(t, finalityUpdateCoversRequired(101, 100, 100))
}

func TestSendWakeCoalesceUpgradesToBeaconCapable(t *testing.T) {
	r := &Relay{trigger: make(chan wakeKind, 1)}
	r.sendWake(wakeOnchainLC)
	r.sendWake(wakeSSE) // coalesces over pending on-chain wake
	select {
	case kind := <-r.trigger:
		assert.Equal(t, wakeSSE, kind)
	default:
		t.Fatal("expected coalesced wake")
	}
	select {
	case <-r.trigger:
		t.Fatal("expected single coalesced wake only")
	default:
	}
}

func TestNewRelayMinMessageRelayerFee(t *testing.T) {
	r := NewRelay(&Config{MinMessageRelayerFeeWei: "1000"}, nil)
	require.NotNil(t, r)
	assert.Equal(t, int64(1000), r.minMessageRelayerFee.Int64())
}
