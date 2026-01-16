package beaconstate

import (
	"sync"
	"time"

	"github.com/ferranbt/fastssz"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type CachedState struct {
	Slot         uint64
	State        state.BeaconState
	Tree         *ssz.Node
	LastAccessed time.Time
}

type CachedProof struct {
	Key       string
	Response  []byte // JSON-encoded proof response
	CreatedAt time.Time
}

type StateCache struct {
	states    map[uint64]*CachedState
	order     []uint64 // LRU order (oldest first)
	maxStates int
	ttl       time.Duration
	mu        sync.RWMutex
}

type ProofCache struct {
	proofs    map[string]*CachedProof
	order     []string // LRU order (oldest first)
	maxProofs int
	ttl       time.Duration
	mu        sync.RWMutex
}

func NewStateCache(maxStates int, ttl time.Duration) *StateCache {
	return &StateCache{
		states:    make(map[uint64]*CachedState),
		order:     make([]uint64, 0, maxStates),
		maxStates: maxStates,
		ttl:       ttl,
	}
}

func NewProofCache(maxProofs int, ttl time.Duration) *ProofCache {
	return &ProofCache{
		proofs:    make(map[string]*CachedProof),
		order:     make([]string, 0, maxProofs),
		maxProofs: maxProofs,
		ttl:       ttl,
	}
}

func (c *StateCache) Get(slot uint64) (*CachedState, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()

	cached, ok := c.states[slot]
	if !ok {
		return nil, false
	}

	// Check TTL
	if time.Since(cached.LastAccessed) > c.ttl {
		c.removeSlot(slot)
		return nil, false
	}

	// Update access time and move to end of LRU
	cached.LastAccessed = time.Now()
	c.moveToEnd(slot)

	return cached, true
}

func (c *StateCache) Put(slot uint64, beaconState state.BeaconState, tree *ssz.Node) {
	c.mu.Lock()
	defer c.mu.Unlock()

	// Evict if at capacity
	for len(c.states) >= c.maxStates {
		c.evictOldest()
	}

	c.states[slot] = &CachedState{
		Slot:         slot,
		State:        beaconState,
		Tree:         tree,
		LastAccessed: time.Now(),
	}
	c.order = append(c.order, slot)
}

func (c *StateCache) moveToEnd(slot uint64) {
	for i, s := range c.order {
		if s == slot {
			c.order = append(c.order[:i], c.order[i+1:]...)
			c.order = append(c.order, slot)
			return
		}
	}
}

func (c *StateCache) removeSlot(slot uint64) {
	delete(c.states, slot)
	for i, s := range c.order {
		if s == slot {
			c.order = append(c.order[:i], c.order[i+1:]...)
			return
		}
	}
}

func (c *StateCache) evictOldest() {
	if len(c.order) == 0 {
		return
	}
	oldest := c.order[0]
	c.order = c.order[1:]
	delete(c.states, oldest)
}

func (c *StateCache) Size() int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return len(c.states)
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
