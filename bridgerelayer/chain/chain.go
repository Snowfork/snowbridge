// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

import (
	"context"

	"golang.org/x/sync/errgroup"
)

type Message struct {
	AppID   [20]byte
	Payload []byte
}

type Chain interface {
	Name() string
	Start(ctx context.Context, eg *errgroup.Group) error
	Stop()
}
