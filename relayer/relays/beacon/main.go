package beacon

import (
	"context"
	"io"
	"net/http"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config *Config
}

func NewRelay(
	config *Config,
) *Relay {
	return &Relay{
		config: config,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	syncer := syncer.New(r.config.Source.Beacon.Endpoint)
	


	return nil
}
