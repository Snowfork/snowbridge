package beacon

import (
	"context"

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
	s := syncer.New(r.config.Source.Beacon.Endpoint)

	header, err := s.GetFinalizedHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch header")
	}

	SyncAggregate, err := s.GetBlockSyncAggregate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block")
	}

	syncCommittee, err := s.GetSyncCommittee()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")
	}

	logrus.WithFields(logrus.Fields{
		"indexes": syncCommittee.Indexes,
	}).Info("fetched sync committee")

	lightClientUpdate := syncer.LightClientUpdate{
		FinalityHeader: header,
		SyncCommittee:  syncCommittee,
		SyncAggregate: SyncAggregate,
	}

	logrus.WithFields(logrus.Fields{
		"lightClientUpdate": lightClientUpdate,
	}).Info("compiled Light Client Update")

	return nil
}
