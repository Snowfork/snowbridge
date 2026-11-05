package state

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// The oracle for the Gloas state layout: a state fetched from a live node must merkleize to
// the state_root that node's own beacon header commits to. It validates the decoder and the
// EIP-7495/7916 merkleization together, since a field parsed at the wrong offset shifts its
// root and the comparison fails.
//
// The state is ~16MB, so it is cached under testdata/ rather than committed. Set
// GLOAS_BEACON_URL to point at another node; the test skips when there is no cache and no
// network, so it never fails for reasons unrelated to the code under test.
const defaultGloasBeaconURL = "https://beacon.plataberget.ethpandaops.io"

func gloasBeaconURL() string {
	if u := os.Getenv("GLOAS_BEACON_URL"); u != "" {
		return u
	}
	return defaultGloasBeaconURL
}

// fetchGloasState returns a cached finalized state and the state_root its header commits to,
// downloading and caching them on first use.
func fetchGloasState(t *testing.T) ([]byte, string) {
	t.Helper()
	sszPath := filepath.Join("testdata", "gloas_state.ssz")
	rootPath := filepath.Join("testdata", "gloas_state.root")

	if raw, err := os.ReadFile(sszPath); err == nil {
		if root, err := os.ReadFile(rootPath); err == nil {
			return raw, strings.TrimSpace(string(root))
		}
	}

	client := &http.Client{Timeout: 3 * time.Minute}
	base := gloasBeaconURL()

	var header struct {
		Data struct {
			Header struct {
				Message struct {
					Slot      string `json:"slot"`
					StateRoot string `json:"state_root"`
				} `json:"message"`
			} `json:"header"`
		} `json:"data"`
	}
	resp, err := client.Get(base + "/eth/v1/beacon/headers/finalized")
	if err != nil {
		t.Skipf("no cached state and no network: %v", err)
	}
	defer resp.Body.Close()
	if err := json.NewDecoder(resp.Body).Decode(&header); err != nil {
		t.Skipf("cannot read finalized header: %v", err)
	}
	slot := header.Data.Header.Message.Slot
	root := header.Data.Header.Message.StateRoot

	req, _ := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/debug/beacon/states/%s", base, slot), nil)
	req.Header.Set("Accept", "application/octet-stream")
	sresp, err := client.Do(req)
	if err != nil {
		t.Skipf("cannot fetch state at slot %s: %v", slot, err)
	}
	defer sresp.Body.Close()
	if sresp.StatusCode != http.StatusOK {
		t.Skipf("node returned %s for state at slot %s", sresp.Status, slot)
	}
	raw, err := io.ReadAll(sresp.Body)
	if err != nil {
		t.Skipf("cannot read state body: %v", err)
	}

	if err := os.MkdirAll("testdata", 0o755); err == nil {
		_ = os.WriteFile(sszPath, raw, 0o644)
		_ = os.WriteFile(rootPath, []byte(root), 0o644)
	}
	t.Logf("fetched slot %s (%d bytes), cached under testdata/", slot, len(raw))
	return raw, root
}

func TestGloasStateRootMatchesNode(t *testing.T) {
	raw, wantRoot := fetchGloasState(t)

	var s BeaconStateGloas
	if err := s.UnmarshalSSZ(raw); err != nil {
		t.Fatalf("decode: %v", err)
	}
	got, err := s.HashTreeRoot()
	if err != nil {
		t.Fatalf("hash tree root: %v", err)
	}
	if "0x"+hex.EncodeToString(got[:]) != wantRoot {
		t.Fatalf("state root mismatch\n got  0x%x\n want %s\n(slot %d)", got, wantRoot, s.Slot)
	}
	t.Logf("slot %d merkleizes to %s", s.Slot, wantRoot)
}
