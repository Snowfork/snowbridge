package syncer

import (
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/types"
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
	SyncCommitteeBits      []byte
	SyncCommitteeSignature []byte
}

type Genesis struct {
	ValidatorsRoot common.Hash
	Time           string
	ForkVersion    []byte
}

type LightClientSnapshot struct {
	Header                     Header
	CurrentSyncCommittee       CurrentSyncCommittee
	CurrentSyncCommitteeBranch []common.Hash
	ValidatorsRoot             common.Hash
}

type FinalizedBlockUpdate struct {
	FinalizedHeader Header
	FinalityBranch  []common.Hash
	SyncAggregate   SyncAggregate
}

type BeaconHeaderScale struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	BodyRoot      types.H256
}

type CurrentSyncCommitteeScale struct {
	Pubkeys         [][48]byte
	AggregatePubkey [48]byte
}

type InitialSync struct {
	Header                     BeaconHeaderScale
	CurrentSyncCommittee       CurrentSyncCommitteeScale
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
}

func (s *Syncer) InitialSync(blockId string) (InitialSync, error) {
	genesis, err := s.Client.GetGenesis()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	snapshot, err := s.Client.GetTrustedLightClientSnapshot()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	slot, err := strconv.ParseUint(snapshot.Data.Header.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return InitialSync{}, err
	}

	proposerIndex, err := strconv.ParseUint(snapshot.Data.Header.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return InitialSync{}, err
	}

	initialSync := InitialSync{
		Header: BeaconHeaderScale{
			Slot:          types.NewU64(slot),
			ProposerIndex: types.NewU64(proposerIndex),
			ParentRoot:    types.NewH256(common.HexToHash(snapshot.Data.Header.ParentRoot).Bytes()),
			StateRoot:     types.NewH256(common.HexToHash(snapshot.Data.Header.StateRoot).Bytes()),
			BodyRoot:      types.NewH256(common.HexToHash(snapshot.Data.Header.BodyRoot).Bytes()),
		},
		ValidatorsRoot: types.NewH256(common.HexToHash(genesis.Data.ValidatorsRoot).Bytes()),
	}

	var syncCommitteePubkeys [][48]byte

	for _, pubkey := range snapshot.Data.CurrentSyncCommittee.Pubkeys {
		publicKey, err := hexStringToPublicKey(pubkey)
		if err != nil {
			logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

			return InitialSync{}, err
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := hexStringToPublicKey(snapshot.Data.CurrentSyncCommittee.AggregatePubkey)
	if err != nil {
		logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

		return InitialSync{}, err
	}

	initialSync.CurrentSyncCommittee = CurrentSyncCommitteeScale{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: syncCommitteeAggPubkey,
	}

	syncCommitteeBranch := []types.H256{}

	for _, proof := range snapshot.Data.CurrentSyncCommitteeBranch {
		syncCommitteeBranch = append(syncCommitteeBranch, types.NewH256(proof.Bytes()))
	}

	initialSync.CurrentSyncCommitteeBranch = syncCommitteeBranch

	logrus.WithFields(logrus.Fields{
		"blockId": blockId,
	}).Info("received initial sync for trusted block, sending for intial sync")

	return initialSync, nil
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

func hexStringToPublicKey(hexString string) ([48]byte, error) {
	var pubkeyBytes [48]byte
	key, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return [48]byte{}, err
	}

	copy(pubkeyBytes[:], key)

	return pubkeyBytes, nil
}
