package parachaincommitment

import (
	"context"
	"errors"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	parachainConnection *parachain.Connection
	ethereumConnection  *ethereum.Connection
	ethereumConfig      *ethereum.Config
	log                 *logrus.Entry
}

func NewListener(parachainConnection *parachain.Connection,
	ethereumConnection *ethereum.Connection,
	ethereumConfig *ethereum.Config,
	log *logrus.Entry,
) *Listener {
	return &Listener{
		parachainConnection: parachainConnection,
		ethereumConnection:  ethereumConnection,
		ethereumConfig:      ethereumConfig,
		log:                 log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {

	blockNumber, err := li.fetchStartBlock()
	if err != nil {
		return nil
	}

	li.catchupMissedCommitments(ctx, blockNumber)

	return nil
}

// Fetch the starting block
func (li *Listener) fetchStartBlock() (uint64, error) {
	header, err := li.parachainConnection.GetFinalizedHeader()
	if err != nil {
		li.log.WithError(err).Error("Failed to fetch hash for starting block")
		return 0, err
	}

	return uint64(header.Number), nil
}

var ErrBlockNotReady = errors.New("required result to be 32 bytes, but got 0")

func getAuxiliaryDigestItem(digest types.Digest) (*chainTypes.AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			return &auxDigestItem, nil
		}
	}
	return nil, nil
}
