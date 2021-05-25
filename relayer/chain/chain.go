// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

import (
	"context"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Message interface{}

// Messages from Substrate
type SubstrateOutboundBasicMessage struct {
	Messages   []substrate.BasicOutboundChannelMessage
	Commitment types.H256
}

type SubstrateOutboundIncentivizedMessage struct {
	Messages   []substrate.IncentivizedOutboundChannelMessage
	Commitment types.H256
}

// Message from ethereum
type EthereumOutboundMessage struct {
	Call string
	Args []interface{}
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
	// TODO: SetReceiver method
	// TODO: SetSender method
}
