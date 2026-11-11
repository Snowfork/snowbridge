package executionV3

import (
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"testing"
	"time"

	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"github.com/snowfork/snowbridge/relayer/relays/gasestimator"
)

// SCALE VersionedXcm V5: RefundSurplus + Definite ether DepositAsset(15e12 → [1;32]) + Wild + SetTopic
var fixtureServiceFeeXCM = mustDecodeHex(
	"0510140d00040201090704000b00f0ab75a40d0001010001010101010101010101010101010101010101010101010101010101010101010d0102080001010001010101010101010101010101010101010101010101010101010101010101012c0909090909090909090909090909090909090909090909090909090909090909",
)

func mustDecodeHex(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return b
}

func TestPassesStaticFeeFloorIncludesServiceFee(t *testing.T) {
	recipient := "0x0101010101010101010101010101010101010101010101010101010101010101"
	r := NewRelay(&Config{
		MinMessageRelayerFeeWei: "10000000000000", // 10e12
		GasEstimation: gasestimator.Config{
			ServiceFeeRecipients: []string{recipient},
		},
	}, nil)
	require.NotNil(t, r)
	require.Len(t, r.serviceFeeRecipients, 1)

	// RelayerFee alone (1e12) is below the floor, but + service fee (15e12) passes.
	ev := &contracts.GatewayOutboundMessageAccepted{
		Payload: contracts.Payload{
			RelayerFee: big.NewInt(1_000_000_000_000),
			Xcm:        contracts.Xcm{Kind: 0, Data: fixtureServiceFeeXCM},
		},
	}
	assert.True(t, r.passesStaticFeeFloor(ev))
	assert.Equal(t, big.NewInt(16_000_000_000_000), r.staticRewardWei(ev))
	assert.Equal(t, big.NewInt(15_000_000_000_000), r.serviceFeeFromPayload(ev))

	// No XCM / no service fee → RelayerFee alone fails the floor.
	evNoFee := &contracts.GatewayOutboundMessageAccepted{
		Payload: contracts.Payload{
			RelayerFee: big.NewInt(1_000_000_000_000),
		},
	}
	assert.False(t, r.passesStaticFeeFloor(evNoFee))
}

func TestPassesStaticFeeFloorSkippedWhenEstimationEnabled(t *testing.T) {
	r := NewRelay(&Config{
		MinMessageRelayerFeeWei: "10000000000000",
		GasEstimation:           gasestimator.Config{Enabled: true},
	}, nil)
	require.True(t, r.gasEstimator.Enabled())

	evLow := &contracts.GatewayOutboundMessageAccepted{
		Payload: contracts.Payload{RelayerFee: big.NewInt(1)},
	}
	assert.True(t, r.passesStaticFeeFloor(evLow))
}

func TestCheckV2ProfitableStaticPathWithoutEstimator(t *testing.T) {
	recipient := "0x0101010101010101010101010101010101010101010101010101010101010101"
	r := NewRelay(&Config{
		MinMessageRelayerFeeWei: "10000000000000",
		GasEstimation: gasestimator.Config{
			Enabled:              false,
			ServiceFeeRecipients: []string{recipient},
		},
	}, nil)

	ev := &contracts.GatewayOutboundMessageAccepted{
		Nonce: 42,
		Payload: contracts.Payload{
			RelayerFee: big.NewInt(1_000_000_000_000),
			Xcm:        contracts.Xcm{Data: fixtureServiceFeeXCM},
		},
	}
	_, econ, ok, err := r.checkV2Profitable(nil, ev, nil, "", 100)
	require.NoError(t, err)
	assert.True(t, ok)
	require.NotNil(t, econ)
	assert.False(t, econ.pastMaxDelay)

	evLow := &contracts.GatewayOutboundMessageAccepted{
		Nonce: 43,
		Payload: contracts.Payload{
			RelayerFee: big.NewInt(1),
		},
		Raw: ethtypes.Log{BlockNumber: 50},
	}
	_, _, ok, err = r.checkV2Profitable(nil, evLow, nil, "", 100)
	require.NoError(t, err)
	assert.False(t, ok)

	t.Setenv("PROFITABILITY_MAX_DELAY_SECS", "120") // 10 eth blocks
	_, econ, ok, err = r.checkV2Profitable(nil, evLow, nil, "", 60)
	require.NoError(t, err)
	assert.True(t, ok)
	require.NotNil(t, econ)
	assert.True(t, econ.pastMaxDelay)
}

func TestV2PastMaxDelay(t *testing.T) {
	t.Setenv("PROFITABILITY_MAX_DELAY_SECS", "120") // 10 blocks at 12s

	ev := &contracts.GatewayOutboundMessageAccepted{Raw: ethtypes.Log{BlockNumber: 100}}
	assert.False(t, v2PastMaxDelay(ev, 109))
	assert.True(t, v2PastMaxDelay(ev, 110))
	assert.False(t, v2PastMaxDelay(nil, 1000))
}

func TestAssetHubDryRunRejectedIsPermanent(t *testing.T) {
	err := fmt.Errorf("%w: FailedToTransactAsset", ErrAssetHubDryRunRejected)
	require.True(t, errors.Is(err, ErrAssetHubDryRunRejected))
	assert.False(t, error_tracking.IsTransientError(err))
}

func TestSkipAssetHubDryRunFailureSoftThenDeadLetter(t *testing.T) {
	t.Setenv("PROFITABILITY_MAX_DELAY_SECS", "120") // 10 eth blocks
	ev := &contracts.GatewayOutboundMessageAccepted{
		Nonce: 99,
		Raw:   ethtypes.Log{BlockNumber: 100},
	}

	ok, err := skipAssetHubDryRunFailure(99, ev, 105, "AssetNotFound")
	require.NoError(t, err)
	assert.False(t, ok)

	ok, err = skipAssetHubDryRunFailure(99, ev, 110, "AssetNotFound")
	assert.False(t, ok)
	require.Error(t, err)
	assert.True(t, errors.Is(err, ErrAssetHubDryRunRejected))
}

func TestSkipAssetHubDryRunFailureDeadLettersWhenMaxDelayUnset(t *testing.T) {
	t.Setenv("PROFITABILITY_MAX_DELAY_SECS", "0")
	ev := &contracts.GatewayOutboundMessageAccepted{
		Nonce: 7,
		Raw:   ethtypes.Log{BlockNumber: 100},
	}
	ok, err := skipAssetHubDryRunFailure(7, ev, 100, "FailedToTransactAsset")
	assert.False(t, ok)
	require.Error(t, err)
	assert.True(t, errors.Is(err, ErrAssetHubDryRunRejected))
}

func TestHandleCandidateErrorDeadLettersAssetHubReject(t *testing.T) {
	path := t.TempDir() + "/dead-nonces.json"
	store, err := error_tracking.NewDeadNonceStore(path)
	require.NoError(t, err)

	r := NewRelay(&Config{GasEstimation: gasestimator.Config{Enabled: false}}, nil)
	r.deadNonces = store

	ahErr := fmt.Errorf("%w: FailedToTransactAsset", ErrAssetHubDryRunRejected)
	require.NoError(t, r.handleCandidateError(1100, ahErr))
	assert.True(t, r.deadNonces.IsDead(1100))

	transient := fmt.Errorf("gas estimation: connection timeout")
	require.Error(t, r.handleCandidateError(1101, transient))
	assert.False(t, r.deadNonces.IsDead(1101))
}

func TestCoversOwnDelivery(t *testing.T) {
	quote := func(reward, delivery int64, pastMaxDelay bool) *v2Economics {
		return &v2Economics{reward: big.NewInt(reward), delivery: big.NewInt(delivery), pastMaxDelay: pastMaxDelay}
	}
	assert.False(t, quote(0, 5, false).coversOwnDelivery())
	assert.True(t, quote(5, 5, false).coversOwnDelivery())
	assert.True(t, quote(0, 5, true).coversOwnDelivery())
	// Estimation off: no per-message prices.
	assert.True(t, (&v2Economics{nonce: 1}).coversOwnDelivery())
}

func TestDeferredNonces(t *testing.T) {
	r := &Relay{deferredV2: make(map[uint64]time.Time)}
	t0 := time.Unix(1_000_000, 0)

	r.deferNonce(7, t0.Add(underpaidRetry))
	r.deferNonce(8, t0.Add(dryRunRetry))
	assert.True(t, r.isDeferred(7, t0.Add(underpaidRetry-time.Second)))
	assert.False(t, r.isDeferred(9, t0))

	assert.False(t, r.isDeferred(7, t0.Add(underpaidRetry)))
	assert.NotContains(t, r.deferredV2, uint64(7))

	r.pruneDeferred([]uint64{9})
	assert.Empty(t, r.deferredV2)
}
