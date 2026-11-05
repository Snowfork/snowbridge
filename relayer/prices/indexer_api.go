package prices

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"

	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/singleflight"
)

const (
	indexerFetchTimeout = 15 * time.Second
	// indexerFailureCooldown is how long to wait before retrying the indexer after
	// a failed fetch, so a down or slow indexer is not re-hit on every lookup.
	indexerFailureCooldown = 5 * time.Minute
	// latestTokenPricesQuery fetches the most recent USD price per token from the
	// snowbridge indexer (snowbridge-subsquid latestTokenPrices resolver).
	latestTokenPricesQuery = `{ latestTokenPrices { tokenAddress priceUSD } }`
)

type indexerTokenPrice struct {
	TokenAddress string  `json:"tokenAddress"`
	PriceUSD     float64 `json:"priceUSD"`
}

type indexerPricesResponse struct {
	Data struct {
		LatestTokenPrices []indexerTokenPrice `json:"latestTokenPrices"`
	} `json:"data"`
	Errors []struct {
		Message string `json:"message"`
	} `json:"errors"`
}

// IndexerPriceClient fetches USD prices from the snowbridge indexer GraphQL API.
// A single query returns the latest price for every known token; the result is
// cached (keyed by DefiLlama key, to match the rest of this package) and
// refreshed once it is older than the TTL.
// The snapshot is immutable once built and is safe for lock-free reads.
type IndexerPriceClient struct {
	url        string
	httpClient *http.Client
	ttl        time.Duration
	// maxStaleness bounds how old a snapshot may be and still be served. Past it, lookups
	// miss so callers fall back to a fresher source (e.g. DefiLlama) during a prolonged
	// indexer outage rather than serving very stale prices. Kept >= ttl so a fresh snapshot
	// is always usable; 0 disables the bound (only the nil check applies).
	maxStaleness time.Duration

	mu           sync.RWMutex
	snapshot     map[string]float64
	fetchedAt    time.Time
	lastFailedAt time.Time

	sfg singleflight.Group
}

func NewIndexerPriceClient(url string, ttl time.Duration) *IndexerPriceClient {
	return &IndexerPriceClient{
		url:          url,
		httpClient:   &http.Client{Timeout: indexerFetchTimeout},
		ttl:          ttl,
		maxStaleness: 2 * ttl,
	}
}

// Price returns the USD price for the given DefiLlama key from the latest indexer
// snapshot, refreshing the snapshot when it is older than the TTL. The second
// return value is false when the indexer has no price for the key, so the caller
// can fall back to another source.
func (c *IndexerPriceClient) Price(ctx context.Context, llamaKey string) (float64, bool) {
	c.mu.RLock()
	fresh := c.snapshot != nil && time.Since(c.fetchedAt) < c.ttl
	cooling := time.Since(c.lastFailedAt) < indexerFailureCooldown
	c.mu.RUnlock()

	// Refresh outside the lock when stale and not backing off; singleflight collapses
	// concurrent refreshes into a single network call.
	if !fresh && !cooling {
		_, _, _ = c.sfg.Do("fetch", func() (interface{}, error) {
			snapshot, err := c.fetchSnapshot(ctx)

			c.mu.Lock()
			defer c.mu.Unlock()
			if err != nil {
				log.WithError(err).Warn("Failed to fetch prices from indexer; backing off and falling back to cache")
				c.lastFailedAt = time.Now()
				return nil, nil
			}
			c.snapshot = snapshot
			c.fetchedAt = time.Now()
			return nil, nil
		})
	}

	c.mu.RLock()
	defer c.mu.RUnlock()
	// Don't serve a snapshot aged past the staleness bound — miss so the caller falls back
	// to a fresher source instead of returning very stale prices during an outage.
	if c.snapshot == nil || (c.maxStaleness > 0 && time.Since(c.fetchedAt) >= c.maxStaleness) {
		return 0, false
	}
	price, ok := c.snapshot[llamaKey]
	if !ok || price <= 0 {
		return 0, false
	}
	return price, true
}

func (c *IndexerPriceClient) fetchSnapshot(ctx context.Context) (map[string]float64, error) {
	reqBody, err := json.Marshal(map[string]string{"query": latestTokenPricesQuery})
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.url, bytes.NewReader(reqBody))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, strings.TrimSpace(string(body)))
	}

	var parsed indexerPricesResponse
	if err := json.Unmarshal(body, &parsed); err != nil {
		return nil, err
	}
	if len(parsed.Errors) > 0 {
		return nil, fmt.Errorf("indexer graphql error: %s", parsed.Errors[0].Message)
	}

	// The indexer keys prices by token address; remap to DefiLlama keys via the
	// shared token config so callers can keep looking prices up by llama key.
	snapshot := make(map[string]float64, len(parsed.Data.LatestTokenPrices))
	for _, row := range parsed.Data.LatestTokenPrices {
		if row.PriceUSD <= 0 {
			continue
		}
		config, ok := TokenPriceConfigByAddress(row.TokenAddress)
		if !ok {
			continue
		}
		snapshot[config.LlamaKey] = row.PriceUSD
	}
	return snapshot, nil
}
