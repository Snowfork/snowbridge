package error_tracking

import (
	"path/filepath"
	"testing"
)

func TestDeadNonceStore_InMemory(t *testing.T) {
	s, err := NewDeadNonceStore("")
	if err != nil {
		t.Fatalf("new in-memory store: %v", err)
	}
	if s.IsDead(264) {
		t.Fatal("nonce should not be dead before being marked")
	}
	if err := s.MarkDead(264, "wasm trap"); err != nil {
		t.Fatalf("mark dead: %v", err)
	}
	if !s.IsDead(264) {
		t.Fatal("nonce should be dead after being marked")
	}
	if err := s.MarkDead(264, "second reason"); err != nil {
		t.Fatalf("re-mark should be no-op: %v", err)
	}
	all := s.All()
	if len(all) != 1 || all[264].Reason != "wasm trap" {
		t.Fatalf("unexpected entries after re-mark: %#v", all)
	}
}

func TestDeadNonceStore_PersistAndReload(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "dead-nonces.json")

	s1, err := NewDeadNonceStore(path)
	if err != nil {
		t.Fatalf("new persistent store: %v", err)
	}
	if err := s1.MarkDead(264, "Fungible(0) decode rejected"); err != nil {
		t.Fatalf("mark dead: %v", err)
	}
	if err := s1.MarkDead(265, "another permanent error"); err != nil {
		t.Fatalf("mark dead: %v", err)
	}

	s2, err := NewDeadNonceStore(path)
	if err != nil {
		t.Fatalf("reload store: %v", err)
	}
	if !s2.IsDead(264) || !s2.IsDead(265) {
		t.Fatal("reloaded store missing entries")
	}
	if s2.IsDead(266) {
		t.Fatal("reloaded store has phantom entry")
	}
	if got := s2.All()[264].Reason; got != "Fungible(0) decode rejected" {
		t.Fatalf("reason not persisted: got %q", got)
	}
}

func TestDeadNonceStore_LoadMissingFileIsEmpty(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "does-not-exist.json")

	s, err := NewDeadNonceStore(path)
	if err != nil {
		t.Fatalf("new store with missing file: %v", err)
	}
	if len(s.All()) != 0 {
		t.Fatal("expected empty store for missing file")
	}
}
