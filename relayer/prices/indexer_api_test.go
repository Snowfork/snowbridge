package prices

import (
	"context"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestIndexerPriceClientMapsTokenAddressToLlamaKey(t *testing.T) {
	var calls int
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls++
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":{"latestTokenPrices":[
			{"tokenAddress":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","priceUSD":1.0},
			{"tokenAddress":"0x0000000000000000000000000000000000000000","priceUSD":2500.0},
			{"tokenAddress":"0xdeadbeef00000000000000000000000000000000","priceUSD":42.0}
		]}}`))
	}))
	defer server.Close()

	client := NewIndexerPriceClient(server.URL, time.Hour)

	usdc, ok := client.Price(context.Background(), "ethereum:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
	require.True(t, ok)
	assert.InDelta(t, 1.0, usdc, 1e-9)

	eth, ok := client.Price(context.Background(), EthLlamaKey)
	require.True(t, ok)
	assert.InDelta(t, 2500.0, eth, 1e-9)

	// Unknown token addresses (not in the shared config) are dropped, so the key misses.
	_, ok = client.Price(context.Background(), "ethereum:0xdeadbeef00000000000000000000000000000000")
	assert.False(t, ok)

	// Snapshot is cached for the TTL: a second lookup must not re-query.
	assert.Equal(t, 1, calls)
}

func TestIndexerPriceClientMissOnError(t *testing.T) {
	var calls int
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls++
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer server.Close()

	client := NewIndexerPriceClient(server.URL, time.Hour)
	_, ok := client.Price(context.Background(), EthLlamaKey)
	assert.False(t, ok)

	// After a failure the client backs off, so a second lookup within the cooldown
	// must not re-hit the indexer (it falls back to other sources instead).
	_, ok = client.Price(context.Background(), EthLlamaKey)
	assert.False(t, ok)
	assert.Equal(t, 1, calls)
}

func TestIndexerPriceClientDropsSnapshotPastStaleness(t *testing.T) {
	var calls int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if atomic.AddInt32(&calls, 1) == 1 {
			w.Header().Set("Content-Type", "application/json")
			_, _ = w.Write([]byte(`{"data":{"latestTokenPrices":[
				{"tokenAddress":"0x0000000000000000000000000000000000000000","priceUSD":2500.0}
			]}}`))
			return
		}
		w.WriteHeader(http.StatusInternalServerError) // indexer down after the first fetch
	}))
	defer server.Close()

	client := NewIndexerPriceClient(server.URL, time.Hour) // ttl 1h, maxStaleness 2h

	price, ok := client.Price(context.Background(), EthLlamaKey)
	require.True(t, ok)
	assert.InDelta(t, 2500.0, price, 1e-9)

	// Age the snapshot past the staleness bound (2h) while the indexer is down.
	client.mu.Lock()
	client.fetchedAt = time.Now().Add(-3 * time.Hour)
	client.lastFailedAt = time.Time{} // ensure not in cooldown, so a refresh is attempted
	client.mu.Unlock()

	// The refresh fails and the stale snapshot must not be served, so the lookup misses
	// and the caller can fall back to a fresher source.
	_, ok = client.Price(context.Background(), EthLlamaKey)
	assert.False(t, ok)
	assert.Equal(t, int32(2), atomic.LoadInt32(&calls)) // a refresh was attempted
}

func TestIndexerPriceClientConcurrentRefreshDedup(t *testing.T) {
	var calls int32
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		atomic.AddInt32(&calls, 1)
		time.Sleep(20 * time.Millisecond) // widen the window so callers overlap on the refresh
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":{"latestTokenPrices":[
			{"tokenAddress":"0x0000000000000000000000000000000000000000","priceUSD":2500.0}
		]}}`))
	}))
	defer server.Close()

	client := NewIndexerPriceClient(server.URL, time.Hour)

	var wg sync.WaitGroup
	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			price, ok := client.Price(context.Background(), EthLlamaKey)
			assert.True(t, ok)
			assert.InDelta(t, 2500.0, price, 1e-9)
		}()
	}
	wg.Wait()

	// singleflight collapses the concurrent refreshes into a single indexer request.
	assert.Equal(t, int32(1), atomic.LoadInt32(&calls))
}
