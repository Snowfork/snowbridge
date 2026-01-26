package beaconstate

import (
	"sync"
	"time"
)

type CachedProof struct {
	Key       string
	Response  []byte // JSON-encoded proof response
	CreatedAt time.Time
}

type ProofCache struct {
	proofs    map[string]*CachedProof
	order     []string // LRU order (oldest first)
	maxProofs int
	ttl       time.Duration
	mu        sync.RWMutex
}

func NewProofCache(maxProofs int, ttl time.Duration) *ProofCache {
	return &ProofCache{
		proofs:    make(map[string]*CachedProof),
		order:     make([]string, 0, maxProofs),
		maxProofs: maxProofs,
		ttl:       ttl,
	}
}

func (c *ProofCache) Get(key string) ([]byte, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()

	cached, ok := c.proofs[key]
	if !ok {
		return nil, false
	}

	// Check TTL
	if time.Since(cached.CreatedAt) > c.ttl {
		c.removeKey(key)
		return nil, false
	}

	// Move to end of LRU
	c.moveToEnd(key)

	return cached.Response, true
}

func (c *ProofCache) Put(key string, response []byte) {
	c.mu.Lock()
	defer c.mu.Unlock()

	// Evict if at capacity
	for len(c.proofs) >= c.maxProofs {
		c.evictOldest()
	}

	c.proofs[key] = &CachedProof{
		Key:       key,
		Response:  response,
		CreatedAt: time.Now(),
	}
	c.order = append(c.order, key)
}

func (c *ProofCache) moveToEnd(key string) {
	for i, k := range c.order {
		if k == key {
			c.order = append(c.order[:i], c.order[i+1:]...)
			c.order = append(c.order, key)
			return
		}
	}
}

func (c *ProofCache) removeKey(key string) {
	delete(c.proofs, key)
	for i, k := range c.order {
		if k == key {
			c.order = append(c.order[:i], c.order[i+1:]...)
			return
		}
	}
}

func (c *ProofCache) evictOldest() {
	if len(c.order) == 0 {
		return
	}
	oldest := c.order[0]
	c.order = c.order[1:]
	delete(c.proofs, oldest)
}

func (c *ProofCache) Size() int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return len(c.proofs)
}
