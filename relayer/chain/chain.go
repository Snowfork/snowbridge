// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

import (
	"context"

	"golang.org/x/sync/errgroup"
)

type Message struct {
	AppID   [20]byte
	Payload interface{}
}

type Header struct {
	HeaderData interface{}
	ProofData  interface{}
}

type Init interface{}

type Chain interface {
	Name() string
	Start(ctx context.Context, eg *errgroup.Group, initOut chan<- Init, initIn <-chan Init) error
	Stop()
	SetReceiver(messages <-chan []Message, headers <-chan Header) error
	SetSender(messages chan<- []Message, headers chan<- Header) error
}
