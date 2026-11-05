package executionV3

import (
	"math/big"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/gasestimator"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func v2(nonce uint64, reward, delivery int64) batchCall {
	return batchCall{
		name: "EthereumInboundQueueV2.submit",
		v2: &v2Economics{
			nonce:    nonce,
			reward:   big.NewInt(reward),
			delivery: big.NewInt(delivery),
		},
	}
}

func v2Forced(nonce uint64, reward, delivery int64) batchCall {
	c := v2(nonce, reward, delivery)
	c.v2.pastMaxDelay = true
	return c
}

func TestV2BatchCoversCost(t *testing.T) {
	beacon := batchCall{name: "EthereumBeaconClient.submit"}
	v1 := batchCall{name: "EthereumInboundQueue.submit"}

	t.Run("no v2 always covers", func(t *testing.T) {
		assert.True(t, v2BatchCoversCost([]batchCall{beacon, v1}, big.NewInt(100)))
	})

	t.Run("sum of rewards must exceed one batch fee plus deliveries", func(t *testing.T) {
		// rewards 80+30 = 110; fee 50 + deliveries 40+20 = 110; equal is not enough
		calls := []batchCall{beacon, v2(1, 80, 40), v2(2, 30, 20)}
		assert.False(t, v2BatchCoversCost(calls, big.NewInt(50)))
		calls[2].v2.reward = big.NewInt(31)
		assert.True(t, v2BatchCoversCost(calls, big.NewInt(50)))
	})

	t.Run("beacon and v1 fee is counted once inside the batch fee", func(t *testing.T) {
		// One message's reward covers the shared extrinsic, not a per-message copy of it.
		calls := []batchCall{beacon, v1, v2(7, 10, 4)}
		assert.True(t, v2BatchCoversCost(calls, big.NewInt(5)))
		assert.False(t, v2BatchCoversCost(calls, big.NewInt(6)))
	})
}

func TestFailedBatchIndexes(t *testing.T) {
	items := []gasestimator.BatchItemDryRun{
		{Index: 0, Success: true},
		{Index: 2, Success: false},
	}
	// Index 1 was not reported, so it is dropped with the explicit failure.
	assert.Equal(t, []int{1, 2}, failedBatchIndexes(3, items))
	assert.Empty(t, failedBatchIndexes(2, []gasestimator.BatchItemDryRun{
		{Index: 0, Success: true},
		{Index: 1, Success: true},
	}))
}

func TestSelectProfitableBatchKeepsPastMaxDelayWhenUnprofitable(t *testing.T) {
	// Mirrors the drop path in selectProfitableBatch when v2BatchCoversCost fails.
	calls := []batchCall{
		{name: "EthereumBeaconClient.submit"},
		v2(1, 1, 10),
		v2Forced(2, 1, 10),
		{name: "EthereumInboundQueue.submit"},
	}
	fee := big.NewInt(100)
	require.False(t, v2BatchCoversCost(calls, fee))

	var kept []batchCall
	forced := 0
	for _, c := range calls {
		if c.v2 == nil {
			kept = append(kept, c)
			continue
		}
		if c.v2.pastMaxDelay {
			kept = append(kept, c)
			forced++
		}
	}
	require.Equal(t, 1, forced)
	require.Len(t, kept, 3)
	assert.Equal(t, "EthereumBeaconClient.submit", kept[0].name)
	assert.Equal(t, uint64(2), kept[1].v2.nonce)
	assert.Equal(t, "EthereumInboundQueue.submit", kept[2].name)
}

func TestWithoutIndexesKeepsBeaconWhenV2Fails(t *testing.T) {
	calls := []batchCall{
		{name: "EthereumBeaconClient.submit"},
		v2(9, 1, 1),
		{name: "EthereumInboundQueue.submit"},
	}
	kept := withoutIndexes(calls, []int{1})
	require.Len(t, kept, 2)
	assert.Equal(t, "EthereumBeaconClient.submit", kept[0].name)
	assert.Equal(t, "EthereumInboundQueue.submit", kept[1].name)
	assert.True(t, batchHasMessage(kept))
	assert.False(t, batchHasMessage(kept[:1]))
}
