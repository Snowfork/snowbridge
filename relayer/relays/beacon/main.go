package beacon

import (
	"context"

	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config *Config
}

func NewRelay(
	config *Config,
) *Relay {
	return &Relay{
		config:  config,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	return nil
}
