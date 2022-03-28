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
	FinalityHeader      syncer.BeaconHeader
	SyncCommittee       syncer.SyncCommittee
	SyncCommitteeBranch []string
	NextSyncCommittee   syncer.SyncCommittee
	SyncAggregate       syncer.SyncAggregate
	FinalityBranch      []string
	PubforkVersion      string
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	s := syncer.New(r.config.Source.Beacon.Endpoint)

	lightClientUpdate, err := buildLightClientUpdateDate(s)
	if err != nil {
		logrus.WithError(err).Error("unable to build light client snapshot")

		return err
	}
	
	logrus.WithFields(logrus.Fields{
		"lightClientUpdate": lightClientUpdate,
	}).Info("compiled Light Client Update")

	return nil
}

func buildLightClientUpdateDate(s syncer.Sync) (LightClientUpdate, error) {
	finalizedCheckpoint, err := s.GetFinalizedCheckpoint();
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return LightClientUpdate{}, err
	}

	snapshot, err := s.GetLightClientSnapshot(finalizedCheckpoint.Data.Finalized.Root)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return LightClientUpdate{}, err
	}

	//header, err := s.GetFinalizedHeader()
	//if err != nil {
	//	logrus.WithError(err).Error("unable to fetch header")
	//
	//	return LightClientUpdate{}, err
	//}

	header, err := snapshot.ToBeaconHeader();
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header")

		return LightClientUpdate{}, err
	}

	SyncAggregate, err := s.GetBlockSyncAggregate(header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block")

		return LightClientUpdate{}, err
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

		return LightClientUpdate{}, err
	}

	nextSyncCommittee, err := s.GetSyncCommittee(nextPeriodEpoch)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")

		return LightClientUpdate{}, err
	}

	pubforkVersion, err := s.GetPubforkVersion(header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch sync committee")

		return LightClientUpdate{}, err
	}

	lightClientUpdate := LightClientUpdate{
		FinalityHeader:      header,
		FinalityBranch:      []string{},
		SyncCommittee:       syncCommittee,
		SyncCommitteeBranch: snapshot.Data.CurrentSyncCommitteeBranch,
		NextSyncCommittee:   nextSyncCommittee,
		SyncAggregate:       SyncAggregate,
		PubforkVersion:      pubforkVersion,
	}

	return lightClientUpdate, nil
}