package syncer

import (
	"encoding/hex"
	"errors"
	"fmt"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

var ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("not found")

const (
	SLOTS_IN_EPOCH                   uint64 = 32
	EPOCHS_PER_SYNC_COMMITTEE_PERIOD uint64 = 256
	SYNC_COMMITTEE_INCREMENT                = 5
)

type Syncer struct {
	Client BeaconClient
	Cache  BeaconCache
}

func New(endpoint string) *Syncer {
	return &Syncer{
		Client: *NewBeaconClient(endpoint),
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

type SyncAggregateScale struct {
	SyncCommitteeBits      []byte
	SyncCommitteeSignature []byte
}

type InitialSync struct {
	Header                     BeaconHeaderScale
	CurrentSyncCommittee       CurrentSyncCommitteeScale
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
}

type SyncCommitteePeriodUpdate struct {
	AttestedHeader          BeaconHeaderScale
	NextSyncCommittee       CurrentSyncCommitteeScale
	NextSyncCommitteeBranch []types.H256
	FinalizedHeader         BeaconHeaderScale
	FinalityBranch          []types.H256
	SyncAggregate           SyncAggregateScale
	ForkVersion             [4]byte
	SyncCommitteePeriod     types.U64
}

type FinalizedHeaderUpdate struct {
	AttestedHeader  BeaconHeaderScale
	FinalizedHeader BeaconHeaderScale
	FinalityBranch  []types.H256
	SyncAggregate   SyncAggregateScale
	ForkVersion     [4]byte
}

type HeaderUpdate struct {
	AttestedHeader  BeaconHeaderScale
	ExecutionHeader types.H256
	SyncAggregate   SyncAggregateScale
	ForkVersion     [4]byte
}

func (s *Syncer) InitialSync(blockId string) (InitialSync, error) {
	genesis, err := s.Client.GetGenesis()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	snapshot, err := s.Client.GetLightClientSnapshot("0x492ab1ad6046dfba5aae0d41bf0a349a3c3609c7c7e39ad9e68fc4e6259b7e88") // 52
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	header, err := beaconHeaderToScale(snapshot.Data.Header)
	if err != nil {

		return InitialSync{}, err
	}

	syncCommittee, err := snapshot.Data.CurrentSyncCommittee.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert sync committee to scale format")

		return InitialSync{}, err
	}

	initialSync := InitialSync{
		Header:                     header,
		CurrentSyncCommittee:       syncCommittee,
		CurrentSyncCommitteeBranch: proofBranchToScale(snapshot.Data.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             types.NewH256(common.HexToHash(genesis.Data.ValidatorsRoot).Bytes()),
	}

	logrus.WithFields(logrus.Fields{
		"blockId": blockId,
	}).Info("received initial sync for trusted block, sending for intial sync")

	return initialSync, nil
}

func (s *Syncer) GetSyncPeriodsToFetch(checkpointSlot uint64) ([]uint64, error) {
	finalizedHeader, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to get header at head")

		return []uint64{}, err
	}

	slot, err := strconv.ParseUint(finalizedHeader.Data.AttestedHeader.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return []uint64{}, err
	}

	currentSyncPeriod := ComputeSyncPeriodAtSlot(slot)
	checkpointSyncPeriod := ComputeSyncPeriodAtSlot(checkpointSlot)

	logrus.WithFields(logrus.Fields{
		"currentSyncPeriod":    currentSyncPeriod,
		"checkpointSyncPeriod": checkpointSyncPeriod,
	}).Info("computed epochs")

	syncPeriodsToFetch := []uint64{}

	for i := checkpointSyncPeriod; i <= currentSyncPeriod; i++ {
		syncPeriodsToFetch = append(syncPeriodsToFetch, i)
	}

	return syncPeriodsToFetch, nil
}

func (s *Syncer) GetSyncCommitteePeriodUpdate(from, to uint64) (SyncCommitteePeriodUpdate, error) {
	committeeUpdates, err := s.Client.GetSyncCommitteePeriodUpdate(from, to)
	if err != nil {
		logrus.WithError(err).Error("unable to build sync committee period update")

		return SyncCommitteePeriodUpdate{}, err
	}

	if len(committeeUpdates.Data) < 1 {
		logrus.WithError(err).Error("no sync committee sync update returned")

		return SyncCommitteePeriodUpdate{}, err
	}

	committeeUpdate := committeeUpdates.Data[0]

	attestedHeader, err := beaconHeaderToScale(committeeUpdate.AttestedHeader)
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return SyncCommitteePeriodUpdate{}, err
	}

	finalizedHeader, err := beaconHeaderToScale(committeeUpdate.FinalizedHeader)
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return SyncCommitteePeriodUpdate{}, err
	}

	nextSyncCommittee, err := committeeUpdate.NextSyncCommittee.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert sync committee to scale format")

		return SyncCommitteePeriodUpdate{}, err
	}

	syncAggregate, err := committeeUpdate.SyncAggregate.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert sync aggregate to scale format")

		return SyncCommitteePeriodUpdate{}, err
	}

	forkVersion, err := hexStringToForkVersion(committeeUpdate.ForkVersion)
	if err != nil {
		logrus.WithError(err).Error("unable convert fork version to scale format")

		return SyncCommitteePeriodUpdate{}, err
	}

	syncCommitteePeriodUpdate := SyncCommitteePeriodUpdate{
		AttestedHeader:          attestedHeader,
		NextSyncCommittee:       nextSyncCommittee,
		NextSyncCommitteeBranch: proofBranchToScale(committeeUpdate.NextSyncCommitteeBranch),
		FinalizedHeader:         finalizedHeader,
		FinalityBranch:          proofBranchToScale(committeeUpdate.FinalityBranch),
		SyncAggregate:           syncAggregate,
		ForkVersion:             forkVersion,
	}

	finalizedHeaderSlot := ComputeSyncPeriodAtSlot(uint64(finalizedHeader.Slot))

	if finalizedHeaderSlot != from {
		return SyncCommitteePeriodUpdate{}, ErrCommitteeUpdateHeaderInDifferentSyncPeriod
	}

	return syncCommitteePeriodUpdate, err
}

func (s *Syncer) GetFinalizedBlockUpdate() (FinalizedHeaderUpdate, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return FinalizedHeaderUpdate{}, err
	}

	attestedHeader, err := beaconHeaderToScale(finalizedUpdate.Data.AttestedHeader)
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return FinalizedHeaderUpdate{}, err
	}

	finalizedHeader, err := beaconHeaderToScale(finalizedUpdate.Data.FinalizedHeader)
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return FinalizedHeaderUpdate{}, err
	}

	currentForkVersion, err := s.Client.GetCurrentForkVersion(uint64(finalizedHeader.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return FinalizedHeaderUpdate{}, err
	}

	forkVersion, err := hexStringToForkVersion(currentForkVersion)
	if err != nil {
		logrus.WithError(err).Error("unable convert fork version to scale format")

		return FinalizedHeaderUpdate{}, err
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse sync aggregate in response")

		return FinalizedHeaderUpdate{}, err
	}

	finalizedHeaderUpdate := FinalizedHeaderUpdate{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  proofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		ForkVersion:     forkVersion,
	}

	return finalizedHeaderUpdate, nil
}

func (s *Syncer) GetHeaderUpdate() (HeaderUpdate, error) {
	latestHeader, err := s.Client.GetLatestHeadUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch latest header checkpoint")

		return HeaderUpdate{}, err
	}

	attestedHeader, err := beaconHeaderToScale(latestHeader.Data.AttestedHeader)
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return HeaderUpdate{}, err
	}

	latestBlock, err := s.Client.GetBeaconBlock(uint64(attestedHeader.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable to fetch latest header checkpoint")

		return HeaderUpdate{}, err
	}

	currentForkVersion, err := s.Client.GetCurrentForkVersion(uint64(attestedHeader.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return HeaderUpdate{}, err
	}

	forkVersion, err := hexStringToForkVersion(currentForkVersion)
	if err != nil {
		logrus.WithError(err).Error("unable convert fork version to scale format")

		return HeaderUpdate{}, err
	}

	syncAggregate, err := latestHeader.Data.SyncAggregate.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse sync aggregate in response")

		return HeaderUpdate{}, err
	}

	headerUpdate := HeaderUpdate{
		AttestedHeader:  attestedHeader,
		ExecutionHeader: types.NewH256(common.HexToHash(latestBlock.Data.Message.Body.ExecutionPayload.BlockHash).Bytes()),
		SyncAggregate:   syncAggregate,
		ForkVersion:     forkVersion,
	}

	return headerUpdate, nil
}

func computeEpochAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func computeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EPOCHS_PER_SYNC_COMMITTEE_PERIOD - (epoch % EPOCHS_PER_SYNC_COMMITTEE_PERIOD))
}

func ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (SLOTS_IN_EPOCH * EPOCHS_PER_SYNC_COMMITTEE_PERIOD)
}

func IsInArray(values []uint64, toCheck uint64) bool {
	for _, value := range values {
		if value == toCheck {
			return true
		}
	}
	return false
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

func hexStringToByteArray(hexString string) ([]byte, error) {
	key, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return []byte{}, err
	}

	return key, nil
}

func hexStringToForkVersion(hexString string) ([4]byte, error) {
	key, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return [4]byte{}, err
	}

	forkVersion4Bytes := [4]byte{}

	copy(forkVersion4Bytes[:], key)

	return forkVersion4Bytes, nil
}

func beaconHeaderToScale(header HeaderResponse) (BeaconHeaderScale, error) {
	slot, err := strconv.ParseUint(header.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeaderScale{}, err
	}

	proposerIndex, err := strconv.ParseUint(header.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeaderScale{}, err
	}

	return BeaconHeaderScale{
		Slot:          types.NewU64(slot),
		ProposerIndex: types.NewU64(proposerIndex),
		ParentRoot:    types.NewH256(common.HexToHash(header.ParentRoot).Bytes()),
		StateRoot:     types.NewH256(common.HexToHash(header.StateRoot).Bytes()),
		BodyRoot:      types.NewH256(common.HexToHash(header.BodyRoot).Bytes()),
	}, nil
}

func (s SyncCommitteeResponse) ToScale() (CurrentSyncCommitteeScale, error) {
	var syncCommitteePubkeys [][48]byte

	for _, pubkey := range s.Pubkeys {
		publicKey, err := hexStringToPublicKey(pubkey)
		if err != nil {
			logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

			return CurrentSyncCommitteeScale{}, err
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := hexStringToPublicKey(s.AggregatePubkey)
	if err != nil {
		logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

		return CurrentSyncCommitteeScale{}, err
	}

	return CurrentSyncCommitteeScale{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: syncCommitteeAggPubkey,
	}, nil
}

func (s SyncAggregateResponse) ToScale() (SyncAggregateScale, error) {
	bits, err := hexStringToByteArray(s.SyncCommitteeBits)
	if err != nil {
		return SyncAggregateScale{}, err
	}

	aggregateSignature, err := hexStringToByteArray(s.SyncCommitteeSignature)
	if err != nil {
		return SyncAggregateScale{}, err
	}

	return SyncAggregateScale{
		SyncCommitteeBits:      bits,
		SyncCommitteeSignature: aggregateSignature,
	}, nil
}

func proofBranchToScale(proofs []common.Hash) []types.H256 {
	syncCommitteeBranch := []types.H256{}

	for _, proof := range proofs {
		syncCommitteeBranch = append(syncCommitteeBranch, types.NewH256(proof.Bytes()))
	}

	return syncCommitteeBranch
}
