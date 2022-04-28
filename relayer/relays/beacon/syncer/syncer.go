package syncer

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

const (
	SLOTS_IN_EPOCH                   uint64 = 32
	EPOCHS_PER_SYNC_COMMITTEE_PERIOD uint64 = 256
	SYNC_COMMITTEE_INCREMENT                = 5
)

type Syncer struct {
	Client BeaconClient
	Cache  BeaconCache
}

func New(endpoint, finalizedUpdateEndpoint string) *Syncer {
	return &Syncer{
		Client: *NewBeaconClient(endpoint, finalizedUpdateEndpoint),
		Cache:  *NewBeaconCache(),
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

type Genesis struct {
	ValidatorsRoot string
	Time           string
	ForkVersion    string
}

type LightClientSnapshot struct {
	Header                     Header
	CurrentSyncCommittee       CurrentSyncCommittee
	CurrentSyncCommitteeBranch []string
	ValidatorsRoot             string
}

type FinalizedBlockUpdate struct {
	FinalizedHeader Header
	FinalityBranch  []string
	SyncAggregate   SyncAggregate
}

func (s *Syncer) InitialSync(blockId string) (LightClientSnapshot, error) {
	genesis, err := s.Client.GetGenesis()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return LightClientSnapshot{}, err
	}

	snapshot, err := s.Client.GetTrustedLightClientSnapshot()
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
		ValidatorsRoot:             genesis.Data.ValidatorsRoot,
	}

	logrus.WithFields(logrus.Fields{
		"lightClientSnapshot": lightClientSnapshot,
	}).Info("compiled light client snapshot, sending for intial sync")

	// TODO make intial_sync dispatchable call

	return lightClientSnapshot, nil
}

func (s *Syncer) SyncCommitteePeriodUpdates(checkpointSlot uint64) error {
	head, err := s.Client.GetHeadHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to get header at head")

		return err
	}

	currentEpoch := computeEpochAtSlot(head.Slot)
	checkpointEpoch := computeEpochAtSlot(checkpointSlot)

	currentSyncPeriod := computeSyncPeriodAtEpoch(currentEpoch)
	checkpointSyncPeriod := computeSyncPeriodAtEpoch(checkpointEpoch)

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

		err = s.syncCommitteeForPeriod(syncPeriodMarker, toPeriod)

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
	head, err = s.Client.GetHeadHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to get header at head")

		return err
	}

	currentUpdatedEpoch := computeEpochAtSlot(head.Slot)
	currentUpdatedSyncPeriod := computeSyncPeriodAtEpoch(currentUpdatedEpoch)

	if currentUpdatedSyncPeriod != toPeriod {
		err = s.syncCommitteeForPeriod(currentUpdatedSyncPeriod, currentUpdatedSyncPeriod)
		if err != nil {
			return err
		}
	}

	return nil
}

func (s *Syncer) syncCommitteeForPeriod(from, to uint64) error {
	committeeUpdates, err := s.Client.GetSyncCommitteePeriodUpdate(from, to)
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

func (s *Syncer) FinalizedBlockUpdate() error {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"finalizedBlockUpdate": finalizedUpdate,
	}).Info("compiled finalized block")

	// TODO make import_finalized_header dispatchable call

	return nil

}

func computeEpochAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func computeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EPOCHS_PER_SYNC_COMMITTEE_PERIOD - (epoch % EPOCHS_PER_SYNC_COMMITTEE_PERIOD))
}

func computeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func computeSyncPeriodAtEpoch(epoch uint64) uint64 {
	return epoch / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
}

func hexToBinaryString(rawHex string) string {
	hex := strings.Replace(rawHex, "0x", "", -1)

	// Chunkify strings into array of strings of 8 characters long (to ParseUint safely below)
	chunkSize := 8

	resultStr := ""
	chunks := []string{}
	for i, r := range hex {
		resultStr = resultStr + string(r)
		if i > 0 && (i+1)%chunkSize == 0 {
			chunks = append(chunks, resultStr)
			resultStr = ""
		}
	}

	// If there was a remainder, add the last string to the chunks as well.
	if resultStr != "" {
		chunks = append(chunks, resultStr)
	}

	// Convert chunks into binary string
	binaryStr := ""
	for _, str := range chunks {
		i, err := strconv.ParseUint(str, 16, 32)
		if err != nil {
			fmt.Printf("%s", err)
		}
		binaryStr = binaryStr + fmt.Sprintf("%b", i)
	}

	return binaryStr
}
