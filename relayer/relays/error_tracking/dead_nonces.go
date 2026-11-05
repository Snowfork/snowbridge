package error_tracking

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"sync"
	"time"

	log "github.com/sirupsen/logrus"
)

// DeadNonceStore tracks message nonces that have failed with a permanent error
// and should never be retried, even across relayer restarts.
//
// When the configured path is non-empty, the store persists synchronously to
// disk via a temp-file-and-rename so writes are atomic. An empty path means
// the store is in-memory only and is reset on each restart; the relayer will
// re-encounter the dead nonce, log the same permanent error and crash again.
//
// To recover a nonce after a fix has been deployed, stop the relayer and
// remove its entry from the on-disk file (or delete the file).
type DeadNonceStore struct {
	mu   sync.RWMutex
	path string
	dead map[uint64]DeadNonceEntry
}

type DeadNonceEntry struct {
	Reason   string    `json:"reason"`
	MarkedAt time.Time `json:"marked_at"`
}

// NewDeadNonceStore loads any existing dead-nonce records from disk.
// An empty path means the store is in-memory only.
func NewDeadNonceStore(path string) (*DeadNonceStore, error) {
	s := &DeadNonceStore{
		path: path,
		dead: make(map[uint64]DeadNonceEntry),
	}
	if path == "" {
		log.Warn("dead-nonce store running in-memory only; permanent errors will be retried after relayer restart")
		return s, nil
	}
	if err := s.load(); err != nil {
		return nil, fmt.Errorf("load dead-nonce store from %s: %w", path, err)
	}
	return s, nil
}

func (s *DeadNonceStore) IsDead(nonce uint64) bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	_, ok := s.dead[nonce]
	return ok
}

// MarkDead records a permanent failure for nonce and persists synchronously.
// Idempotent: re-marking an already-dead nonce is a no-op and does not rewrite
// the file. Returns an error only if persistence fails; the in-memory record
// is updated regardless so the caller can proceed.
func (s *DeadNonceStore) MarkDead(nonce uint64, reason string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, exists := s.dead[nonce]; exists {
		return nil
	}
	s.dead[nonce] = DeadNonceEntry{
		Reason:   reason,
		MarkedAt: time.Now().UTC(),
	}
	log.WithFields(log.Fields{
		"event":  "dead_nonce",
		"nonce":  nonce,
		"reason": reason,
	}).Error("marked nonce as permanently dead; will be skipped on subsequent runs")
	if s.path == "" {
		return nil
	}
	return s.persistLocked()
}

// All returns a copy of all currently-recorded dead nonces.
func (s *DeadNonceStore) All() map[uint64]DeadNonceEntry {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make(map[uint64]DeadNonceEntry, len(s.dead))
	for k, v := range s.dead {
		out[k] = v
	}
	return out
}

func (s *DeadNonceStore) load() error {
	data, err := os.ReadFile(s.path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return err
	}
	if len(data) == 0 {
		return nil
	}
	var raw map[string]DeadNonceEntry
	if err := json.Unmarshal(data, &raw); err != nil {
		return fmt.Errorf("decode dead-nonce file: %w", err)
	}
	for k, v := range raw {
		nonce, err := strconv.ParseUint(k, 10, 64)
		if err != nil {
			return fmt.Errorf("invalid nonce key %q in dead-nonce file: %w", k, err)
		}
		s.dead[nonce] = v
	}
	log.WithField("count", len(s.dead)).Info("loaded dead-nonce store")
	return nil
}

func (s *DeadNonceStore) persistLocked() error {
	out := make(map[string]DeadNonceEntry, len(s.dead))
	for k, v := range s.dead {
		out[strconv.FormatUint(k, 10)] = v
	}
	data, err := json.MarshalIndent(out, "", "  ")
	if err != nil {
		return fmt.Errorf("encode dead-nonce store: %w", err)
	}
	dir := filepath.Dir(s.path)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("create dead-nonce store dir: %w", err)
	}
	tmp, err := os.CreateTemp(dir, ".dead-nonces-*.tmp")
	if err != nil {
		return fmt.Errorf("create temp file: %w", err)
	}
	tmpPath := tmp.Name()
	renamed := false
	defer func() {
		if !renamed {
			_ = os.Remove(tmpPath)
		}
	}()

	if _, err := tmp.Write(data); err != nil {
		_ = tmp.Close()
		return fmt.Errorf("write temp file: %w", err)
	}
	if err := tmp.Sync(); err != nil {
		_ = tmp.Close()
		return fmt.Errorf("sync temp file: %w", err)
	}
	if err := tmp.Close(); err != nil {
		return fmt.Errorf("close temp file: %w", err)
	}
	if err := os.Rename(tmpPath, s.path); err != nil {
		return fmt.Errorf("rename temp file: %w", err)
	}
	renamed = true
	return nil
}
