package ofac

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"
)

func TestIsBannedDisabled(t *testing.T) {
	o := New(Config{Enabled: false})
	banned, err := o.IsBanned("0xabc", []string{"0xdef"})
	if err != nil || banned {
		t.Fatalf("disabled OFAC should no-op, got banned=%v err=%v", banned, err)
	}
}

func TestIsBannedCachesCleanAndSanctioned(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		if r.Method != http.MethodPost {
			t.Errorf("method=%s", r.Method)
		}
		var req []screenRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Errorf("decode: %v", err)
			http.Error(w, "bad json", 400)
			return
		}
		out := make([]screenResponse, 0, len(req))
		for _, item := range req {
			out = append(out, screenResponse{
				Address:      item.Address,
				IsSanctioned: normalizeAddress(item.Address) == "0xbad",
			})
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(out)
	}))
	defer srv.Close()

	o := New(Config{Enabled: true, CacheTTL: time.Hour})
	o.endpoint = srv.URL
	o.client = srv.Client()

	banned, err := o.IsBanned("0xBAD", []string{"0xGood"})
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if !banned {
		t.Fatal("expected banned source")
	}
	if calls.Load() != 1 {
		t.Fatalf("calls=%d want 1", calls.Load())
	}

	banned, err = o.IsBanned("0xbad", []string{"0xgood"})
	if err != nil || !banned {
		t.Fatalf("cached sanctioned miss: banned=%v err=%v", banned, err)
	}
	if calls.Load() != 1 {
		t.Fatalf("sanctioned cache should skip network, calls=%d", calls.Load())
	}

	banned, err = o.IsBanned("", []string{"0xGood"})
	if err != nil || banned {
		t.Fatalf("clean address: banned=%v err=%v", banned, err)
	}
	if calls.Load() != 1 {
		t.Fatalf("clean cache should skip network, calls=%d", calls.Load())
	}
}

func TestIsBannedCleanCacheExpires(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_, _ = w.Write([]byte(`[{"address":"0xgood","isSanctioned":false}]`))
	}))
	defer srv.Close()

	now := time.Unix(1_700_000_000, 0)
	o := New(Config{Enabled: true, CacheTTL: time.Minute})
	o.endpoint = srv.URL
	o.client = srv.Client()
	o.now = func() time.Time { return now }

	if _, err := o.IsBanned("0xgood", nil); err != nil {
		t.Fatal(err)
	}
	now = now.Add(time.Minute + time.Second)
	if _, err := o.IsBanned("0xgood", nil); err != nil {
		t.Fatal(err)
	}
	if calls.Load() != 2 {
		t.Fatalf("calls=%d want 2 after TTL expiry", calls.Load())
	}
}

func TestIsBannedHTTPErrorFailClosed(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "Access restricted", http.StatusForbidden)
	}))
	defer srv.Close()

	o := New(Config{Enabled: true})
	o.endpoint = srv.URL
	o.client = srv.Client()

	banned, err := o.IsBanned("0xabc", nil)
	if err == nil || !banned {
		t.Fatalf("want fail-closed banned+err, got banned=%v err=%v", banned, err)
	}
}

func TestIsBannedBatchesAddresses(t *testing.T) {
	var got []screenRequest
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(&got)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_, _ = w.Write([]byte(`[{"address":"0xaaa","isSanctioned":false},{"address":"0xbbb","isSanctioned":false}]`))
	}))
	defer srv.Close()

	o := New(Config{Enabled: true})
	o.endpoint = srv.URL
	o.client = srv.Client()

	banned, err := o.IsBanned("0xAaA", []string{"0xBbB", "0xaaa"})
	if err != nil || banned {
		t.Fatalf("banned=%v err=%v", banned, err)
	}
	if len(got) != 2 {
		t.Fatalf("batch size=%d want 2 (deduped)", len(got))
	}
}

func TestIsBannedPartialResponseFailClosed(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		// Empty body must not mark requested addresses as clean.
		_, _ = w.Write([]byte(`[]`))
	}))
	defer srv.Close()

	o := New(Config{Enabled: true})
	o.endpoint = srv.URL
	o.client = srv.Client()

	banned, err := o.IsBanned("0xabc", nil)
	if err == nil || !banned {
		t.Fatalf("want fail-closed banned+err on missing address, got banned=%v err=%v", banned, err)
	}
}

func TestIsBannedRetriesTransientHTTP(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		n := calls.Add(1)
		if n < 3 {
			http.Error(w, "slow down", http.StatusTooManyRequests)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`[{"address":"0xabc","isSanctioned":false}]`))
	}))
	defer srv.Close()

	o := New(Config{Enabled: true})
	o.endpoint = srv.URL
	o.client = srv.Client()

	banned, err := o.IsBanned("0xabc", nil)
	if err != nil || banned {
		t.Fatalf("banned=%v err=%v", banned, err)
	}
	if calls.Load() != 3 {
		t.Fatalf("calls=%d want 3", calls.Load())
	}
}
