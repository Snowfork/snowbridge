package beaconstate

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

func TestParseSlotParam(t *testing.T) {
	tests := []struct {
		name     string
		url      string
		wantSlot uint64
		wantErr  bool
	}{
		{
			name:     "valid slot",
			url:      "/test?slot=12345",
			wantSlot: 12345,
			wantErr:  false,
		},
		{
			name:     "missing slot",
			url:      "/test",
			wantSlot: 0,
			wantErr:  true,
		},
		{
			name:     "invalid slot format",
			url:      "/test?slot=invalid",
			wantSlot: 0,
			wantErr:  true,
		},
		{
			name:     "negative slot",
			url:      "/test?slot=-1",
			wantSlot: 0,
			wantErr:  true,
		},
		{
			name:     "zero slot",
			url:      "/test?slot=0",
			wantSlot: 0,
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tt.url, nil)
			slot, err := parseSlotParam(req)

			if (err != nil) != tt.wantErr {
				t.Errorf("parseSlotParam() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if slot != tt.wantSlot {
				t.Errorf("parseSlotParam() = %v, want %v", slot, tt.wantSlot)
			}
		})
	}
}

func TestHashesToHexStrings(t *testing.T) {
	input := [][]byte{
		{0x01, 0x02, 0x03},
		{0xab, 0xcd, 0xef},
	}

	result := hashesToHexStrings(input)

	if len(result) != 2 {
		t.Errorf("hashesToHexStrings() returned %d items, want 2", len(result))
	}

	if result[0] != "0x010203" {
		t.Errorf("hashesToHexStrings()[0] = %s, want 0x010203", result[0])
	}

	if result[1] != "0xabcdef" {
		t.Errorf("hashesToHexStrings()[1] = %s, want 0xabcdef", result[1])
	}
}

func TestWriteError(t *testing.T) {
	w := httptest.NewRecorder()
	writeError(w, http.StatusBadRequest, "test error")

	if w.Code != http.StatusBadRequest {
		t.Errorf("writeError() status = %d, want %d", w.Code, http.StatusBadRequest)
	}

	contentType := w.Header().Get("Content-Type")
	if contentType != "application/json" {
		t.Errorf("writeError() Content-Type = %s, want application/json", contentType)
	}

	var response ErrorResponse
	if err := json.Unmarshal(w.Body.Bytes(), &response); err != nil {
		t.Errorf("writeError() response not valid JSON: %v", err)
	}

	if response.Error != "test error" {
		t.Errorf("writeError() error message = %s, want 'test error'", response.Error)
	}
}

func TestService_HandleHealth(t *testing.T) {
	// Create a minimal service for health check
	cache := NewProofCache(100, 1*time.Hour)
	s := &Service{
		config: &Config{
			Beacon: beaconconf.BeaconConfig{
				Endpoint: "http://localhost:5052",
			},
		},
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	w := httptest.NewRecorder()

	s.handleHealth(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("handleHealth() status = %d, want %d", w.Code, http.StatusOK)
	}

	var response HealthResponse
	if err := json.Unmarshal(w.Body.Bytes(), &response); err != nil {
		t.Errorf("handleHealth() response not valid JSON: %v", err)
	}

	if response.Status != "healthy" {
		t.Errorf("handleHealth() status = %s, want 'healthy'", response.Status)
	}

	if response.BeaconEndpoint != "http://localhost:5052" {
		t.Errorf("handleHealth() endpoint = %s, want 'http://localhost:5052'", response.BeaconEndpoint)
	}
}

func TestService_HandleBlockRootProof_NotCached(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)
	s := &Service{
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/block-root?slot=12345", nil)
	w := httptest.NewRecorder()

	s.handleBlockRootProof(w, req)

	if w.Code != http.StatusServiceUnavailable {
		t.Errorf("handleBlockRootProof() status = %d, want %d", w.Code, http.StatusServiceUnavailable)
	}

	retryAfter := w.Header().Get("Retry-After")
	if retryAfter != "5" {
		t.Errorf("handleBlockRootProof() Retry-After = %s, want '5'", retryAfter)
	}

	var response ErrorResponse
	if err := json.Unmarshal(w.Body.Bytes(), &response); err != nil {
		t.Errorf("handleBlockRootProof() response not valid JSON: %v", err)
	}

	if response.Error != "proof not ready, please retry" {
		t.Errorf("handleBlockRootProof() error = %s, want 'proof not ready, please retry'", response.Error)
	}
}

func TestService_HandleBlockRootProof_Cached(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)

	// Pre-populate cache
	cachedResponse := BlockRootProofResponse{
		Slot:             12345,
		Leaf:             "0xabcd",
		Proof:            []string{"0x1234"},
		GeneralizedIndex: 100,
		BlockRoots:       []string{"0x5678"},
	}
	jsonResponse, _ := json.Marshal(cachedResponse)
	cache.Put("block-root:12345", jsonResponse)

	s := &Service{
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/block-root?slot=12345", nil)
	w := httptest.NewRecorder()

	s.handleBlockRootProof(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("handleBlockRootProof() status = %d, want %d", w.Code, http.StatusOK)
	}

	var response BlockRootProofResponse
	if err := json.Unmarshal(w.Body.Bytes(), &response); err != nil {
		t.Errorf("handleBlockRootProof() response not valid JSON: %v", err)
	}

	if response.Slot != 12345 {
		t.Errorf("handleBlockRootProof() slot = %d, want 12345", response.Slot)
	}
}

func TestService_HandleBlockRootProof_MissingSlot(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)
	s := &Service{
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/block-root", nil)
	w := httptest.NewRecorder()

	s.handleBlockRootProof(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("handleBlockRootProof() status = %d, want %d", w.Code, http.StatusBadRequest)
	}
}

func TestService_HandleFinalizedHeaderProof_NotCached(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)
	s := &Service{
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/finalized-header?slot=12345", nil)
	w := httptest.NewRecorder()

	s.handleFinalizedHeaderProof(w, req)

	if w.Code != http.StatusServiceUnavailable {
		t.Errorf("handleFinalizedHeaderProof() status = %d, want %d", w.Code, http.StatusServiceUnavailable)
	}
}

func TestService_HandleSyncCommitteeProof_NotCached(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)
	s := &Service{
		proofCache: cache,
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/sync-committee?slot=12345&period=current", nil)
	w := httptest.NewRecorder()

	s.handleSyncCommitteeProof(w, req)

	if w.Code != http.StatusServiceUnavailable {
		t.Errorf("handleSyncCommitteeProof() status = %d, want %d", w.Code, http.StatusServiceUnavailable)
	}
}

func TestService_HandleSyncCommitteeProof_DefaultPeriod(t *testing.T) {
	cache := NewProofCache(100, 1*time.Hour)

	// Pre-populate cache with "current" period (default)
	cachedResponse := SyncCommitteeProofResponse{
		Slot:             12345,
		Leaf:             "0xabcd",
		Proof:            []string{"0x1234"},
		GeneralizedIndex: 100,
		Pubkeys:          []string{"0x5678"},
		AggregatePubkey:  "0x9abc",
	}
	jsonResponse, _ := json.Marshal(cachedResponse)
	cache.Put("sync-committee:12345:current", jsonResponse)

	s := &Service{
		proofCache: cache,
	}

	// Request without period parameter - should default to "current"
	req := httptest.NewRequest(http.MethodGet, "/v1/proofs/sync-committee?slot=12345", nil)
	w := httptest.NewRecorder()

	s.handleSyncCommitteeProof(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("handleSyncCommitteeProof() status = %d, want %d", w.Code, http.StatusOK)
	}
}
