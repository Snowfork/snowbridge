package beaconstate

import (
	"testing"
	"time"
)

func TestProofCache_PutAndGet(t *testing.T) {
	cache := NewProofCache(10, 1*time.Hour)

	key := "test-key"
	value := []byte(`{"test": "value"}`)

	cache.Put(key, value)

	got, ok := cache.Get(key)
	if !ok {
		t.Error("Get() returned false, want true")
	}
	if string(got) != string(value) {
		t.Errorf("Get() = %s, want %s", got, value)
	}
}

func TestProofCache_GetMissing(t *testing.T) {
	cache := NewProofCache(10, 1*time.Hour)

	_, ok := cache.Get("nonexistent")
	if ok {
		t.Error("Get() returned true for nonexistent key, want false")
	}
}

func TestProofCache_TTLExpiration(t *testing.T) {
	cache := NewProofCache(10, 50*time.Millisecond)

	key := "expiring-key"
	value := []byte(`{"test": "value"}`)

	cache.Put(key, value)

	// Should be available immediately
	_, ok := cache.Get(key)
	if !ok {
		t.Error("Get() returned false immediately after Put, want true")
	}

	// Wait for TTL to expire
	time.Sleep(60 * time.Millisecond)

	// Should be expired now
	_, ok = cache.Get(key)
	if ok {
		t.Error("Get() returned true after TTL expired, want false")
	}
}

func TestProofCache_LRUEviction(t *testing.T) {
	cache := NewProofCache(3, 1*time.Hour)

	// Fill cache
	cache.Put("key1", []byte("value1"))
	cache.Put("key2", []byte("value2"))
	cache.Put("key3", []byte("value3"))

	// Access key2 and key3 to make them more recently used
	// Order after puts: [key1, key2, key3]
	cache.Get("key2") // Order: [key1, key3, key2]
	cache.Get("key3") // Order: [key1, key2, key3]

	// Add one more - should evict key1 (least recently used)
	cache.Put("key4", []byte("value4"))

	// key1 should be evicted (oldest/least recently used)
	if _, ok := cache.Get("key1"); ok {
		t.Error("key1 should have been evicted")
	}

	// Others should still be present
	if _, ok := cache.Get("key2"); !ok {
		t.Error("key2 should still be present")
	}
	if _, ok := cache.Get("key3"); !ok {
		t.Error("key3 should still be present")
	}
	if _, ok := cache.Get("key4"); !ok {
		t.Error("key4 should still be present")
	}
}

func TestProofCache_Size(t *testing.T) {
	cache := NewProofCache(10, 1*time.Hour)

	if cache.Size() != 0 {
		t.Errorf("Size() = %d, want 0", cache.Size())
	}

	cache.Put("key1", []byte("value1"))
	if cache.Size() != 1 {
		t.Errorf("Size() = %d, want 1", cache.Size())
	}

	cache.Put("key2", []byte("value2"))
	if cache.Size() != 2 {
		t.Errorf("Size() = %d, want 2", cache.Size())
	}
}

func TestProofCache_OverwriteExistingKey(t *testing.T) {
	cache := NewProofCache(10, 1*time.Hour)

	cache.Put("key", []byte("value1"))
	cache.Put("key", []byte("value2"))

	got, ok := cache.Get("key")
	if !ok {
		t.Error("Get() returned false, want true")
	}
	if string(got) != "value2" {
		t.Errorf("Get() = %s, want value2", got)
	}

	// Size should account for the duplicate (this is current behavior)
	// Note: Current implementation doesn't deduplicate, which could be improved
}
