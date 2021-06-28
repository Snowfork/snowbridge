// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

import (
	"context"

	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"golang.org/x/sync/errgroup"
)

type Message interface{}

// Messages from Parachain
type ParachainOutboundBasicMessage struct {
	Messages   []parachain.BasicOutboundChannelMessage
	Commitment types.H256
}

type ParachainOutboundIncentivizedMessage struct {
	Messages   []parachain.IncentivizedOutboundChannelMessage
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
