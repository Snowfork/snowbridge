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

type Header struct {
	Slot          string
	ProposerIndex string
	ParentRoot    string
	StateRoot     string
	BodyRoot      string
}

type CurrentSyncCommittee struct {
	Pubkeys          []string
	AggregatePubkeys string
}

type LightClientSnapshot struct {
	Header                     Header
	CurrentSyncCommittee       CurrentSyncCommittee
	CurrentSyncCommitteeBranch []string
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	s := syncer.New(r.config.Source.Beacon.Endpoint)

	lightClientSnapshot, err := buildSnapShotUpdate(s)
	if err != nil {
		logrus.WithError(err).Error("unable to build light client snapshot")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"lightClientSnapshot": lightClientSnapshot,
	}).Info("compiled Light Client Snapshot")

	return nil
}

func buildSnapShotUpdate(s syncer.Sync) (LightClientSnapshot, error) {
	checkpoint, err := s.GetHeadCheckpoint()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return LightClientSnapshot{}, err
	}

	logrus.WithFields(logrus.Fields{
		"checkpoint": checkpoint,
	}).Info("fetched finalized checkpoint")

	snapshot, err := s.GetLightClientSnapshot(checkpoint.Data.Finalized.Root)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return LightClientSnapshot{}, err
	}

	lightClientSnapshot := LightClientSnapshot{
		Header: Header{
			Slot:          snapshot.Data.Header.Slot,
			ProposerIndex: snapshot.Data.Header.ProposerIndex,
			ParentRoot:    snapshot.Data.Header.ParentRoot,
			StateRoot:     snapshot.Data.Header.StateRoot,
			BodyRoot:      snapshot.Data.Header.BodyRoot,
		},
		CurrentSyncCommittee: CurrentSyncCommittee{
			Pubkeys:          snapshot.Data.CurrentSyncCommittee.Pubkeys,
			AggregatePubkeys: snapshot.Data.CurrentSyncCommittee.AggregatePubkeys,
		},
		CurrentSyncCommitteeBranch: snapshot.Data.CurrentSyncCommitteeBranch,
	}

	return lightClientSnapshot, nil
}
