package beacon

import (
	"context"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"

	//"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"golang.org/x/sync/errgroup"
)

const SYNC_COMMITTEE_INCREMENT = 5

type Relay struct {
	config *Config
	syncer syncer.Syncer
	//keypair  *sr25519.Keypair
	paraconn *parachain.Connection
}

func NewRelay(
	config *Config,
) *Relay {
	return &Relay{
		config: config,
	}
}

type Header struct {
	Slot          uint64
	ProposerIndex uint64
	ParentRoot    common.Hash
	StateRoot     common.Hash
	BodyRoot      common.Hash
}

type CurrentSyncCommittee struct {
	Pubkeys          []string
	AggregatePubkeys string
}

type SyncAggregate struct {
	SyncCommitteeBits      string
	SyncCommitteeSignature string
}

type LightClientSnapshot struct {
	Header                     Header
	CurrentSyncCommittee       CurrentSyncCommittee
	CurrentSyncCommitteeBranch []string
}

type FinalizedBlockUpdate struct {
	FinalizedHeader Header
	FinalityBranch  []string
	SyncAggregate   SyncAggregate
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	r.syncer = syncer.New(r.config.Source.Beacon.Endpoint)
	//r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())

	// Get an initial snapshot of the chain from a verified block
	lightClientSnapshot, err := r.buildSnapShotUpdate("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	if err != nil {
		logrus.WithError(err).Error("unable to build light client snapshot")

		return err
	}

	err = r.SyncCommitteePeriodUpdates(lightClientSnapshot.Header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to sync committee updates")

		return err
	}

	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	err = r.buildFinalizedBlockUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to build light client snapshot")

		return err
	}

	return nil
}

func (r *Relay) buildSnapShotUpdate(blockId string) (LightClientSnapshot, error) {
	snapshot, err := r.syncer.GetTrustedLightClientSnapshot()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return LightClientSnapshot{}, err
	}

	slot, err := strconv.ParseUint(snapshot.Data.Header.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return LightClientSnapshot{}, err
	}

	proposerIndex, err := strconv.ParseUint(snapshot.Data.Header.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return LightClientSnapshot{}, err
	}

	lightClientSnapshot := LightClientSnapshot{
		Header: Header{
			Slot:          slot,
			ProposerIndex: proposerIndex,
			ParentRoot:    common.HexToHash(snapshot.Data.Header.ParentRoot),
			StateRoot:     common.HexToHash(snapshot.Data.Header.StateRoot),
			BodyRoot:      common.HexToHash(snapshot.Data.Header.BodyRoot),
		},
		CurrentSyncCommittee: CurrentSyncCommittee{
			Pubkeys:          snapshot.Data.CurrentSyncCommittee.Pubkeys,
			AggregatePubkeys: snapshot.Data.CurrentSyncCommittee.AggregatePubkey,
		},
		CurrentSyncCommitteeBranch: snapshot.Data.CurrentSyncCommitteeBranch,
	}

	logrus.WithFields(logrus.Fields{
		"lightClientSnapshot": lightClientSnapshot,
	}).Info("compiled light client snapshot, sending for intial sync")

	// TODO make intial_sync dispatchable call

	return lightClientSnapshot, nil
}

func (r *Relay) SyncCommitteePeriodUpdates(checkpointSlot uint64) error {
	head, err := r.syncer.GetHeadHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to get header at head")

		return err
	}

	currentEpoch := syncer.ComputeEpochAtSlot(head.Slot)
	checkpointEpoch := syncer.ComputeEpochAtSlot(checkpointSlot)

	currentSyncPeriod := syncer.ComputeSyncPeriodAtEpoch(currentEpoch)
	checkpointSyncPeriod := syncer.ComputeSyncPeriodAtEpoch(checkpointEpoch)

	syncPeriodMarker := checkpointSyncPeriod

	logrus.WithFields(logrus.Fields{
		"currentEpoch":         currentEpoch,
		"checkpointEpoch":      checkpointEpoch,
		"currentSyncPeriod":    currentSyncPeriod,
		"checkpointSyncPeriod": checkpointSyncPeriod,
	}).Info("computed epochs")

	var toPeriod uint64
	// Incrementally move the chain forward by fetching an update per sync period and sending that to the parachain
	for syncPeriodMarker < currentSyncPeriod {
		logrus.WithFields(logrus.Fields{
			"syncPeriodMarker":  syncPeriodMarker,
			"currentSyncPeriod": currentSyncPeriod,
		}).Info("checking...")

		toPeriod := syncPeriodMarker + SYNC_COMMITTEE_INCREMENT

		if toPeriod > currentSyncPeriod {
			toPeriod = currentSyncPeriod
		}

		err = r.syncCommitteeForPeriod(syncPeriodMarker, toPeriod)

		syncPeriodMarker = toPeriod + 1
		if err != nil {
			logrus.WithError(err).WithFields(logrus.Fields{
				"from": syncPeriodMarker,
				"to":   toPeriod,
			}).Error("unable to get sync committeee update for period")

			return err
		}
	}

	// Check corner case where the sync period may have progressed while processing sync committee updates.
	head, err = r.syncer.GetHeadHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to get header at head")

		return err
	}

	currentUpdatedEpoch := syncer.ComputeEpochAtSlot(head.Slot)
	currentUpdatedSyncPeriod := syncer.ComputeSyncPeriodAtEpoch(currentUpdatedEpoch)

	if currentUpdatedSyncPeriod != toPeriod {
		err = r.syncCommitteeForPeriod(currentUpdatedSyncPeriod, currentUpdatedSyncPeriod)
		if err != nil {
			return err
		}
	}

	return nil
}

func (r *Relay) syncCommitteeForPeriod(from, to uint64) error {
	committeeUpdates, err := r.syncer.GetSyncCommitteePeriodUpdate(from, to)
	if err != nil {
		logrus.WithError(err).Error("unable to build sync committee period update")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"from":                from,
		"to":                  to,
		"syncCommitteeUpdate": committeeUpdates,
	}).Info("fetched sync committee for period")

	// TODO make sync_committee_period_update dispatchable call

	return nil
}

func (r *Relay) buildFinalizedBlockUpdate() error {
	checkpoint, err := r.syncer.GetHeadCheckpoint()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return err
	}

	header, err := r.syncer.GetHeader(checkpoint.Data.Finalized.Root)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch header")

		return err
	}

	block, err := r.syncer.GetBeaconBlock(header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch header")

		return err
	}

	/*
		proofs, err := s.GetFinalizedCheckpointProofs(header.StateRoot.String())
		if err != nil {
			logrus.WithError(err).Error("unable to fetch proofs")

			return FinalizedBlockUpdate{}, err
		}

		logrus.WithFields(logrus.Fields{
			"proofs": proofs,
		}).Info("fetched proofs")*/

	update := FinalizedBlockUpdate{
		FinalizedHeader: Header{
			Slot:          header.Slot,
			ProposerIndex: header.ProposerIndex,
			ParentRoot:    header.ParentRoot,
			StateRoot:     header.StateRoot,
			BodyRoot:      header.BodyRoot,
		},
		FinalityBranch: []string{},
		SyncAggregate: SyncAggregate{
			SyncCommitteeBits:      block.Data.Message.Body.SyncAggregate.SyncCommitteeBits,
			SyncCommitteeSignature: block.Data.Message.Body.SyncAggregate.SyncCommitteeSignature,
		},
		// TODO add Pubfork version
	}

	logrus.WithFields(logrus.Fields{
		"finalizedBlockUpdate": update,
	}).Info("compiled finalized block")

	// TODO make import_finalized_header dispatchable call

	return nil

}
