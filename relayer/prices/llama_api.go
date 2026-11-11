package prices

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	log "github.com/sirupsen/logrus"
)

const (
	llamaPricesURL   = "https://coins.llama.fi/prices/current"
	llamaChunkSize   = 15
	llamaChunkDelay  = 300 * time.Millisecond
	llamaFetchTimout = 15 * time.Second
	llamaMaxRetries  = 2
	// defaultLlamaCacheTTL bounds how long a fetched price is reused before it is
	// refetched. Without it, a long-lived client (e.g. the process-lived gas
	// estimator) would freeze prices for the lifetime of the relayer. Override
	// with the LLAMA_PRICE_CACHE_TTL_SECONDS environment variable.
	defaultLlamaCacheTTL = 4 * time.Hour
	llamaCacheTTLEnvKey  = "LLAMA_PRICE_CACHE_TTL_SECONDS"
	// indexerAPIURLEnvKey points at the snowbridge indexer GraphQL endpoint. When
	// set, prices are read from the indexer first and the DefiLlama cache is used
	// only as a fallback. Leave unset to query DefiLlama directly.
	indexerAPIURLEnvKey = "PRICE_INDEXER_API_URL"
)

// llamaCacheTTLFromEnv reads the cache TTL from LLAMA_PRICE_CACHE_TTL_SECONDS,
// falling back to defaultLlamaCacheTTL when unset or invalid. A value of 0
// disables cache reuse (every lookup refetches).
func llamaCacheTTLFromEnv() time.Duration {
	valStr := os.Getenv(llamaCacheTTLEnvKey)
	if valStr == "" {
		return defaultLlamaCacheTTL
	}
	seconds, err := strconv.ParseInt(valStr, 10, 64)
	if err != nil || seconds < 0 {
		log.Warnf("Invalid value for %s: %q, using default %s", llamaCacheTTLEnvKey, valStr, defaultLlamaCacheTTL)
		return defaultLlamaCacheTTL
	}
	return time.Duration(seconds) * time.Second
}

type llamaCoinResult struct {
	Price float64 `json:"price"`
}

type llamaPricesResponse struct {
	Coins map[string]llamaCoinResult `json:"coins"`
}

type cachedPrice struct {
	price     float64
	fetchedAt time.Time
}

type LlamaPriceClient struct {
	httpClient *http.Client
	mu         sync.RWMutex
	cache      map[string]cachedPrice
	cacheTTL   time.Duration
	// indexer, when configured, is the preferred price source; the DefiLlama
	// cache below is used as a fallback when the indexer has no price or is down.
	indexer *IndexerPriceClient
}

func (c *LlamaPriceClient) cacheGet(key string) (cachedPrice, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	entry, ok := c.cache[key]
	return entry, ok
}

func (c *LlamaPriceClient) cacheSet(key string, price float64) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.cache[key] = cachedPrice{price: price, fetchedAt: time.Now()}
}

func NewLlamaPriceClient() *LlamaPriceClient {
	client := &LlamaPriceClient{
		httpClient: &http.Client{Timeout: llamaFetchTimout},
		cache:      make(map[string]cachedPrice),
		cacheTTL:   llamaCacheTTLFromEnv(),
	}
	if url := os.Getenv(indexerAPIURLEnvKey); url != "" {
		client.indexer = NewIndexerPriceClient(url, client.cacheTTL)
	}
	return client
}

func (c *LlamaPriceClient) SetPrice(llamaKey string, price float64) {
	c.cacheSet(llamaKey, price)
}

func (c *LlamaPriceClient) Price(ctx context.Context, llamaKey string) (float64, error) {
	if llamaKey == "" {
		return 0, fmt.Errorf("empty llama key")
	}

	// Prefer the indexer when configured; keep the cache warm so it can serve as a
	// fallback if the indexer later becomes unavailable.
	if c.indexer != nil {
		if price, ok := c.indexer.Price(ctx, llamaKey); ok {
			c.cacheSet(llamaKey, price)
			return price, nil
		}
	}

	// Tokens with no live feed resolve to their configured fixed price.
	if price, ok := FixedPriceByLlamaKey(llamaKey); ok {
		return price, nil
	}

	if entry, ok := c.cacheGet(llamaKey); ok && time.Since(entry.fetchedAt) < c.cacheTTL {
		return entry.price, nil
	}

	prices, err := c.fetchPrices(ctx, []string{llamaKey})
	if err != nil {
		return 0, err
	}
	price, ok := extractLlamaPrice(prices, llamaKey)
	if !ok {
		return 0, fmt.Errorf("no price returned for %s", llamaKey)
	}
	c.cacheSet(llamaKey, price)
	return price, nil
}

func (c *LlamaPriceClient) fetchPrices(ctx context.Context, coinKeys []string) (map[string]float64, error) {
	if len(coinKeys) == 0 {
		return map[string]float64{}, nil
	}

	uniqueKeys := dedupeStrings(coinKeys)
	allPrices := make(map[string]float64, len(uniqueKeys))

	for i := 0; i < len(uniqueKeys); i += llamaChunkSize {
		end := i + llamaChunkSize
		if end > len(uniqueKeys) {
			end = len(uniqueKeys)
		}
		chunk := uniqueKeys[i:end]

		url := llamaPricesURL + "/" + strings.Join(chunk, ",")
		response, err := c.fetchWithRetry(ctx, url)
		if err != nil {
			return nil, fmt.Errorf("fetch llama prices chunk: %w", err)
		}

		for key, coin := range response.Coins {
			if coin.Price > 0 {
				allPrices[key] = coin.Price
				c.cacheSet(key, coin.Price)
			}
		}

		if end < len(uniqueKeys) {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(llamaChunkDelay):
			}
		}
	}

	return allPrices, nil
}

func (c *LlamaPriceClient) fetchWithRetry(ctx context.Context, url string) (*llamaPricesResponse, error) {
	var lastErr error
	for attempt := 0; attempt <= llamaMaxRetries; attempt++ {
		req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
		if err != nil {
			return nil, err
		}

		resp, err := c.httpClient.Do(req)
		if err != nil {
			lastErr = err
		} else {
			body, readErr := io.ReadAll(resp.Body)
			resp.Body.Close()
			if readErr != nil {
				lastErr = readErr
			} else if resp.StatusCode != http.StatusOK {
				lastErr = fmt.Errorf("HTTP %d: %s", resp.StatusCode, strings.TrimSpace(string(body)))
			} else {
				var parsed llamaPricesResponse
				if decodeErr := json.Unmarshal(body, &parsed); decodeErr != nil {
					lastErr = decodeErr
				} else {
					return &parsed, nil
				}
			}
		}

		if attempt < llamaMaxRetries {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(time.Duration(400*(attempt+1)) * time.Millisecond):
			}
		}
	}
	return nil, lastErr
}

func extractLlamaPrice(coins map[string]float64, llamaKey string) (float64, bool) {
	if price, ok := coins[llamaKey]; ok && price > 0 {
		return price, true
	}

	keyLower := strings.ToLower(llamaKey)
	for key, price := range coins {
		if strings.ToLower(key) == keyLower && price > 0 {
			return price, true
		}
	}
	return 0, false
}

func dedupeStrings(values []string) []string {
	seen := make(map[string]struct{}, len(values))
	out := make([]string, 0, len(values))
	for _, value := range values {
		if value == "" {
			continue
		}
		if _, ok := seen[value]; ok {
			continue
		}
		seen[value] = struct{}{}
		out = append(out, value)
	}
	return out
}

func RawAmountToUsd(amount *big.Int, decimals uint8, priceUsd float64) float64 {
	if amount == nil || amount.Sign() <= 0 || priceUsd <= 0 {
		return 0
	}

	divisor := new(big.Float).SetInt(new(big.Int).Exp(big.NewInt(10), big.NewInt(int64(decimals)), nil))
	amountFloat := new(big.Float).SetInt(amount)
	amountFloat.Quo(amountFloat, divisor)
	humanAmount, _ := amountFloat.Float64()
	return humanAmount * priceUsd
}

func FeeWeiToUsd(feeWei *big.Int, ethPriceUsd float64) float64 {
	if feeWei == nil || feeWei.Sign() <= 0 || ethPriceUsd <= 0 {
		return 0
	}

	feeFloat := new(big.Float).SetInt(feeWei)
	feeFloat.Quo(feeFloat, big.NewFloat(1e18))
	feeEth, _ := feeFloat.Float64()
	return feeEth * ethPriceUsd
}
