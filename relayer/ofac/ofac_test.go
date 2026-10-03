package ofac

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestIsOFACListedRejectsUnexpectedStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusTooManyRequests)
	}))
	t.Cleanup(server.Close)

	checker := New(true, "test-key")
	checker.endpoint = server.URL

	banned, err := checker.isOFACListed("0x1234")
	require.Error(t, err)
	require.True(t, banned)
}

func TestIsOFACListedUsesClientTimeout(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		time.Sleep(100 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
	}))
	t.Cleanup(server.Close)

	checker := New(true, "test-key")
	checker.endpoint = server.URL
	checker.client.Timeout = 10 * time.Millisecond

	banned, err := checker.isOFACListed("0x1234")
	require.Error(t, err)
	require.True(t, banned)
}
