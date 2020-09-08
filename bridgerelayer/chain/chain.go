package chain

import (
	"context"

	"golang.org/x/sync/errgroup"
)

type Message struct {
	AppID   [32]byte
	Payload []byte
}

type Chain interface {
	Name() string
	Start(ctx context.Context, eg *errgroup.Group) error
	Stop()
}

// TODO: These are interim standins/hacks which will be removed once
// https://github.com/Snowfork/polkadot-ethereum/issues/61 lands.
var Erc20AppID = [32]byte{
	1, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
}

var EthAppID = [32]byte{
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
}
