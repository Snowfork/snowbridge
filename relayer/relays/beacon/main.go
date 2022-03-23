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

type LightClientUpdate struct {
	FinalityHeader    syncer.BeaconHeader
	SyncCommittee     syncer.SyncCommittee
	NextSyncCommittee syncer.SyncCommittee
	SyncAggregate     syncer.SyncAggregate
	FinalityBranch    []string
	PubforkVersion    string
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	s := syncer.New(r.config.Source.Beacon.Endpoint)

	header, err := s.GetFinalizedHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch header")

		return err
	}

	SyncAggregate, err := s.GetBlockSyncAggregate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block")

		return err
	}

	currentEpoch := syncer.ComputeEpochAtSlot(header.Slot)
	nextPeriodEpoch := syncer.ComputeEpochForNextPeriod(currentEpoch)

	logrus.WithFields(logrus.Fields{
		"currentEpoch":    currentEpoch,
		"nextPeriodEpoch": nextPeriodEpoch,
	}).Info("computed epochs")

	syncCommittee, err := s.GetSyncCommittee(currentEpoch)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")

		return err
	}

	nextSyncCommittee, err := s.GetSyncCommittee(nextPeriodEpoch)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")

		return err
	}

	pubforkVersion, err := s.GetPubforkVersion(header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")

		return err
	}

	lightClientUpdate := syncer.LightClientUpdate{
		FinalityHeader:    header,
		FinalityBranch:    []string{},
		SyncCommittee:     syncCommittee,
		NextSyncCommittee: nextSyncCommittee,
		SyncAggregate:     SyncAggregate,
		PubforkVersion:    pubforkVersion,
	}

	logrus.WithFields(logrus.Fields{
		"lightClientUpdate": lightClientUpdate,
	}).Info("compiled Light Client Update")

	return nil
}
