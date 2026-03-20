// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"errors"
	"net"
	"strings"
)

// LikelyTransientRPCError reports whether err often indicates a flaky primary RPC
// (timeouts, gateway errors) where retrying via FallbackEndpoint may succeed.
func LikelyTransientRPCError(err error) bool {
	if err == nil {
		return false
	}
	if errors.Is(err, context.DeadlineExceeded) {
		return true
	}
	var netErr net.Error
	if errors.As(err, &netErr) && netErr.Timeout() {
		return true
	}
	msg := err.Error()
	low := strings.ToLower(msg)
	return strings.Contains(msg, "504") ||
		strings.Contains(low, "gateway timeout") ||
		strings.Contains(low, "deadline exceeded") ||
		strings.Contains(low, "i/o timeout") ||
		strings.Contains(low, "connection reset")
}
