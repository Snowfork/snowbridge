package api

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestSubscribeFinalizedCheckpoint(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")
		w.WriteHeader(http.StatusOK)

		// Send a comment
		fmt.Fprintf(w, ": welcome\n\n")
		// Send a different event
		fmt.Fprintf(w, "event: other_event\ndata: {}\n\n")
		// Send finalized checkpoint event
		fmt.Fprintf(w, "event: finalized_checkpoint\ndata: {\"block\":\"0x1111111111111111111111111111111111111111111111111111111111111111\",\"state\":\"0x2222222222222222222222222222222222222222222222222222222222222222\",\"epoch\":\"45\"}\n\n")
	}))
	defer server.Close()

	client := NewBeaconClient([]string{server.URL})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	eventsCh, errCh, err := client.SubscribeFinalizedCheckpoint(ctx)
	if err != nil {
		t.Fatalf("failed to subscribe: %v", err)
	}

	select {
	case event := <-eventsCh:
		if event.Epoch != "45" {
			t.Errorf("expected epoch 45, got %s", event.Epoch)
		}
		if event.Block != "0x1111111111111111111111111111111111111111111111111111111111111111" {
			t.Errorf("unexpected block root: %s", event.Block)
		}
	case err := <-errCh:
		t.Fatalf("received stream error: %v", err)
	case <-ctx.Done():
		t.Fatal("timeout waiting for event")
	}
}

// TestSubscribeFinalizedCheckpoint_MidStreamDisconnect verifies that when the
// stream drops after delivering an event, the reader surfaces a non-nil error
// on errCh so the driver can reconnect.
func TestSubscribeFinalizedCheckpoint_MidStreamDisconnect(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/event-stream")
		w.WriteHeader(http.StatusOK)

		// Deliver one valid event, flush it to the client, then return so the
		// connection closes mid-stream.
		fmt.Fprintf(w, "event: finalized_checkpoint\ndata: {\"block\":\"0x1111111111111111111111111111111111111111111111111111111111111111\",\"state\":\"0x2222222222222222222222222222222222222222222222222222222222222222\",\"epoch\":\"45\"}\n\n")
		if f, ok := w.(http.Flusher); ok {
			f.Flush()
		}
	}))
	defer server.Close()

	client := NewBeaconClient([]string{server.URL})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	eventsCh, errCh, err := client.SubscribeFinalizedCheckpoint(ctx)
	if err != nil {
		t.Fatalf("failed to subscribe: %v", err)
	}

	// The disconnect must produce both the delivered event and a subsequent
	// stream error. Loop rather than relying on select ordering.
	var gotEvent, gotErr bool
	for !gotEvent || !gotErr {
		select {
		case event := <-eventsCh:
			if event.Epoch != "45" {
				t.Errorf("expected epoch 45, got %s", event.Epoch)
			}
			gotEvent = true
		case streamErr := <-errCh:
			if streamErr == nil {
				t.Fatal("expected a non-nil stream error on disconnect, got nil")
			}
			gotErr = true
		case <-ctx.Done():
			t.Fatalf("timeout waiting for disconnect (gotEvent=%v gotErr=%v)", gotEvent, gotErr)
		}
	}
}

// TestSubscribeFinalizedCheckpoint_MalformedData verifies that a finalized
// checkpoint frame with invalid JSON is skipped (no event emitted, no panic)
// and that the stream keeps processing subsequent well-formed frames.
func TestSubscribeFinalizedCheckpoint_MalformedData(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/event-stream")
		w.WriteHeader(http.StatusOK)

		// Malformed finalized_checkpoint frame: invalid JSON in data.
		fmt.Fprintf(w, "event: finalized_checkpoint\ndata: {not-valid-json}\n\n")
		// Followed by a well-formed one.
		fmt.Fprintf(w, "event: finalized_checkpoint\ndata: {\"block\":\"0x3333333333333333333333333333333333333333333333333333333333333333\",\"state\":\"0x4444444444444444444444444444444444444444444444444444444444444444\",\"epoch\":\"99\"}\n\n")
	}))
	defer server.Close()

	client := NewBeaconClient([]string{server.URL})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	eventsCh, errCh, err := client.SubscribeFinalizedCheckpoint(ctx)
	if err != nil {
		t.Fatalf("failed to subscribe: %v", err)
	}

	// The malformed frame must be dropped, so the first (and only) event
	// delivered is the well-formed epoch 99 one.
	select {
	case event := <-eventsCh:
		if event.Epoch != "99" {
			t.Errorf("expected epoch 99 (malformed frame should be skipped), got %s", event.Epoch)
		}
		if event.Block != "0x3333333333333333333333333333333333333333333333333333333333333333" {
			t.Errorf("unexpected block root: %s", event.Block)
		}
	case err := <-errCh:
		t.Fatalf("received stream error before valid event: %v", err)
	case <-ctx.Done():
		t.Fatal("timeout waiting for valid event after malformed frame")
	}
}
