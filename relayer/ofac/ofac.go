package ofac

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"

	log "github.com/sirupsen/logrus"
)

const (
	defaultEndpoint = "https://api.trmlabs.com/public/v1/sanctions/screening"
	defaultCacheTTL = 6 * time.Hour
	httpTimeout     = 5 * time.Second
)

// Config configures the TRM free Sanctions Screening client.
// ApiKey is optional: the public endpoint works without a key (low rate limits);
// a TRM sanctions API key raises those limits.
type Config struct {
	Enabled  bool
	ApiKey   string
	CacheTTL time.Duration
}

type OFAC struct {
	enabled  bool
	apiKey   string
	endpoint string
	cacheTTL time.Duration
	client   *http.Client
	now      func() time.Time

	mu    sync.Mutex
	cache map[string]cacheEntry
}

type cacheEntry struct {
	banned    bool
	expiresAt time.Time // zero means no expiry (used for sanctioned hits)
}

type screenRequest struct {
	Address string `json:"address"`
}

type screenResponse struct {
	Address      string `json:"address"`
	IsSanctioned bool   `json:"isSanctioned"`
}

func New(cfg Config) *OFAC {
	ttl := cfg.CacheTTL
	if ttl <= 0 {
		ttl = defaultCacheTTL
	}
	return &OFAC{
		enabled:  cfg.Enabled,
		apiKey:   cfg.ApiKey,
		endpoint: defaultEndpoint,
		cacheTTL: ttl,
		client:   &http.Client{Timeout: httpTimeout},
		now:      time.Now,
		cache:    make(map[string]cacheEntry),
	}
}

// IsBanned reports whether source or any destination is sanctioned.
// On provider errors it returns (true, err) so callers fail closed.
func (o *OFAC) IsBanned(source string, destinations []string) (bool, error) {
	if o == nil || !o.enabled {
		return false, nil
	}

	addrs := make([]string, 0, 1+len(destinations))
	if source != "" {
		addrs = append(addrs, source)
	}
	for _, destination := range destinations {
		if destination != "" {
			addrs = append(addrs, destination)
		}
	}
	if len(addrs) == 0 {
		return false, nil
	}

	results, err := o.screen(addrs)
	if err != nil {
		return true, err
	}

	if source != "" && results[normalizeAddress(source)] {
		log.WithField("source", source).Warn("found ofac banned source address")
		return true, nil
	}
	for _, destination := range destinations {
		if destination != "" && results[normalizeAddress(destination)] {
			log.WithField("destination", destination).Warn("found ofac banned destination address")
			return true, nil
		}
	}
	return false, nil
}

func (o *OFAC) screen(addresses []string) (map[string]bool, error) {
	out := make(map[string]bool, len(addresses))
	var missing []string
	seen := make(map[string]struct{}, len(addresses))

	now := o.now()
	o.mu.Lock()
	for _, addr := range addresses {
		key := normalizeAddress(addr)
		if _, ok := seen[key]; ok {
			continue
		}
		seen[key] = struct{}{}
		if entry, ok := o.cache[key]; ok && (entry.expiresAt.IsZero() || now.Before(entry.expiresAt)) {
			out[key] = entry.banned
			continue
		}
		missing = append(missing, addr)
	}
	o.mu.Unlock()

	if len(missing) == 0 {
		return out, nil
	}

	fetched, err := o.fetch(missing)
	if err != nil {
		return nil, err
	}

	now = o.now()
	o.mu.Lock()
	for key, banned := range fetched {
		out[key] = banned
		entry := cacheEntry{banned: banned}
		if banned {
			// Sanctioned hits stay cached for the process lifetime.
		} else {
			entry.expiresAt = now.Add(o.cacheTTL)
		}
		o.cache[key] = entry
	}
	o.mu.Unlock()

	return out, nil
}

func (o *OFAC) fetch(addresses []string) (map[string]bool, error) {
	body := make([]screenRequest, 0, len(addresses))
	for _, addr := range addresses {
		body = append(body, screenRequest{Address: addr})
	}
	payload, err := json.Marshal(body)
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest(http.MethodPost, o.endpoint, bytes.NewReader(payload))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Accept", "application/json")
	req.Header.Set("Content-Type", "application/json")
	if o.apiKey != "" {
		// TRM organization API keys use HTTP Basic with an empty password.
		req.SetBasicAuth(o.apiKey, "")
	}

	resp, err := o.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("TRM sanctions API HTTP %d: %s", resp.StatusCode, truncateForLog(respBody))
	}

	var results []screenResponse
	if err := json.Unmarshal(respBody, &results); err != nil {
		return nil, fmt.Errorf("decode TRM sanctions response: %w", err)
	}

	out := make(map[string]bool, len(addresses))
	for _, addr := range addresses {
		out[normalizeAddress(addr)] = false
	}
	for _, item := range results {
		out[normalizeAddress(item.Address)] = item.IsSanctioned
	}
	return out, nil
}

func normalizeAddress(address string) string {
	if strings.HasPrefix(address, "0x") || strings.HasPrefix(address, "0X") {
		return strings.ToLower(address)
	}
	return address
}

func truncateForLog(body []byte) string {
	const max = 180
	s := string(body)
	if len(s) <= max {
		return s
	}
	return s[:max] + "…"
}
