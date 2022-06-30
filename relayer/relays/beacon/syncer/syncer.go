package syncer

import (
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"strconv"
	"strings"

	ssz "github.com/ferranbt/fastssz"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer/scale"
)

var ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("not found")

const (
	SlotsInEpoch                 uint64 = 32
	EpochsPerSyncCommitteePeriod uint64 = 256
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

type InitialSync struct {
	Header                     scale.BeaconHeader
	CurrentSyncCommittee       scale.CurrentSyncCommittee
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
}

type SyncCommitteePeriodUpdate struct {
	AttestedHeader          scale.BeaconHeader
	NextSyncCommittee       scale.CurrentSyncCommittee
	NextSyncCommitteeBranch []types.H256
	FinalizedHeader         scale.BeaconHeader
	FinalityBranch          []types.H256
	SyncAggregate           scale.SyncAggregate
	ForkVersion             [4]byte
	SyncCommitteePeriod     types.U64
}

type FinalizedHeaderUpdate struct {
	AttestedHeader  scale.BeaconHeader
	FinalizedHeader scale.BeaconHeader
	FinalityBranch  []types.H256
	SyncAggregate   scale.SyncAggregate
	ForkVersion     [4]byte
}

type HeaderUpdate struct {
	Block         scale.BeaconBlock
	BlockBodyRoot types.H256
	SyncAggregate scale.SyncAggregate
	ForkVersion   [4]byte
}

func (s *Syncer) InitialSync(blockId string) (InitialSync, error) {
	genesis, err := s.Client.GetGenesis()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	snapshot, err := s.Client.GetLightClientSnapshot(blockId)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch snapshot")

		return InitialSync{}, err
	}

	header, err := snapshot.Data.Header.ToScale()
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

	attestedHeader, err := committeeUpdate.AttestedHeader.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return SyncCommitteePeriodUpdate{}, err
	}

	finalizedHeader, err := committeeUpdate.FinalizedHeader.ToScale()
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

func (s *Syncer) GetFinalizedUpdate() (FinalizedHeaderUpdate, common.Hash, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse beacon header in response")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	currentForkVersion, err := s.Client.GetCurrentForkVersion(uint64(finalizedHeader.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable to fetch current fork version")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	blockRoot, err := s.Client.GetBeaconBlockRoot(uint64(finalizedHeader.Slot)) // TODO can compute this ourselves with SSZ
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block root")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	forkVersion, err := hexStringToForkVersion(currentForkVersion)
	if err != nil {
		logrus.WithError(err).Error("unable convert fork version to scale format")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable to parse sync aggregate in response")

		return FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	finalizedHeaderUpdate := FinalizedHeaderUpdate{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  proofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		ForkVersion:     forkVersion,
	}

	return finalizedHeaderUpdate, blockRoot, nil
}

func (s *Syncer) GetHeaderUpdate(blockRoot common.Hash) (HeaderUpdate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch latest header checkpoint")

		return HeaderUpdate{}, err
	}

	header, err := s.Client.GetHeader(blockRoot.Hex())
	if err != nil {
		logrus.WithError(err).Error("unable to fetch latest header checkpoint")

		return HeaderUpdate{}, err
	}

	blockScale, err := block.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert block to scale format")

		return HeaderUpdate{}, err
	}

	blockRoot, err = s.Client.GetBeaconBlockRoot(uint64(blockScale.Slot)) // TODO can compute this ourselves with SSZ
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block root")

		return HeaderUpdate{}, err
	}

	currentForkVersion, err := s.Client.GetCurrentForkVersion(uint64(blockScale.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable to fetch finalized checkpoint")

		return HeaderUpdate{}, err
	}

	forkVersion, err := hexStringToForkVersion(currentForkVersion)
	if err != nil {
		logrus.WithError(err).Error("unable convert fork version to scale format")

		return HeaderUpdate{}, err
	}

	headerUpdate := HeaderUpdate{
		Block:         blockScale,
		BlockBodyRoot: types.NewH256(header.BodyRoot.Bytes()),
		ForkVersion:   forkVersion,
	}

	return headerUpdate, nil
}

func (s *Syncer) GetSyncAggregate(blockRoot common.Hash) (scale.SyncAggregate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		logrus.WithError(err).Error("unable to fetch block")

		return scale.SyncAggregate{}, err
	}

	blockScale, err := block.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert block to scale format")

		return scale.SyncAggregate{}, err
	}

	return blockScale.Body.SyncAggregate, nil
}

func (s *Syncer) GetSyncAggregateForSlot(slot uint64) (scale.SyncAggregate, error) {
	block, err := s.Client.GetBeaconBlockBySlot(slot)
	if err != nil {
		if errors.Is(err, ErrNotFound) {
			safeguard := 0
			for err != nil && safeguard < 20 {
				logrus.WithField("slot", slot).Info("no block at slot, skipping to next")
				block, err = s.Client.GetBeaconBlockBySlot(slot + 1)
				safeguard = safeguard + 1
			}
		}

		if err != nil {
			logrus.WithError(err).Error("unable to fetch block")

			return scale.SyncAggregate{}, err
		}
	}

	blockScale, err := block.ToScale()
	if err != nil {
		logrus.WithError(err).Error("unable convert block to scale format")

		return scale.SyncAggregate{}, err
	}

	return blockScale.Body.SyncAggregate, nil
}

func computeEpochAtSlot(slot uint64) uint64 {
	return slot / SlotsInEpoch
}

func computeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EpochsPerSyncCommitteePeriod - (epoch % EpochsPerSyncCommitteePeriod))
}

func ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (SlotsInEpoch * EpochsPerSyncCommitteePeriod)
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
	hexString := strings.Replace(rawHex, "0x", "", -1)

	// Chunkify strings into array of strings of 8 characters long (to ParseUint safely below)
	chunkSize := 8

	resultStr := ""
	chunks := []string{}
	for i, r := range hexString {
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
	result, ok := new(big.Int).SetString(hexString[2:], 16)
	if !ok {
		return []byte{}, nil
	}

	return result.Bytes(), nil
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

func (h HeaderResponse) ToScale() (scale.BeaconHeader, error) {
	slot, err := strconv.ParseUint(h.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return scale.BeaconHeader{}, err
	}

	proposerIndex, err := strconv.ParseUint(h.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return scale.BeaconHeader{}, err
	}

	return scale.BeaconHeader{
		Slot:          types.NewU64(slot),
		ProposerIndex: types.NewU64(proposerIndex),
		ParentRoot:    types.NewH256(common.HexToHash(h.ParentRoot).Bytes()),
		StateRoot:     types.NewH256(common.HexToHash(h.StateRoot).Bytes()),
		BodyRoot:      types.NewH256(common.HexToHash(h.BodyRoot).Bytes()),
	}, nil
}

func (s SyncCommitteeResponse) ToScale() (scale.CurrentSyncCommittee, error) {
	var syncCommitteePubkeys [][48]byte

	for _, pubkey := range s.Pubkeys {
		publicKey, err := hexStringToPublicKey(pubkey)
		if err != nil {
			logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

			return scale.CurrentSyncCommittee{}, err
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := hexStringToPublicKey(s.AggregatePubkey)
	if err != nil {
		logrus.WithError(err).Error("unable convert sync committee pubkey to byte array")

		return scale.CurrentSyncCommittee{}, err
	}

	return scale.CurrentSyncCommittee{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: syncCommitteeAggPubkey,
	}, nil
}

func (s SyncAggregateResponse) ToScale() (scale.SyncAggregate, error) {
	bits, err := hexStringToByteArray(s.SyncCommitteeBits)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	aggregateSignature, err := hexStringToByteArray(s.SyncCommitteeSignature)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	return scale.SyncAggregate{
		SyncCommitteeBits:      bits,
		SyncCommitteeSignature: aggregateSignature,
	}, nil
}

func (b BeaconBlockResponse) ToScale() (scale.BeaconBlock, error) {
	dataMessage := b.Data.Message

	slot, err := toUint64(dataMessage.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")
		return scale.BeaconBlock{}, err
	}

	proposerIndex, err := toUint64(dataMessage.ProposerIndex)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")
		return scale.BeaconBlock{}, err
	}

	body := dataMessage.Body

	syncAggregate, err := body.SyncAggregate.ToScale()
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	proposerSlashings := []scale.ProposerSlashing{}

	for _, proposerSlashing := range body.ProposerSlashings {
		proposerSlashingScale, err := proposerSlashing.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		proposerSlashings = append(proposerSlashings, proposerSlashingScale)
	}

	attesterSlashings := []scale.AttesterSlashing{}

	for _, attesterSlashing := range body.AttesterSlashings {
		attesterSlashingScale, err := attesterSlashing.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		attesterSlashings = append(attesterSlashings, attesterSlashingScale)
	}

	attestations := []scale.Attestation{}

	for _, attestation := range body.Attestations {
		attestationScale, err := attestation.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		attestations = append(attestations, attestationScale)
	}

	deposits := []scale.Deposit{}

	for _, deposit := range body.Deposits {
		depositScale, err := deposit.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		deposits = append(deposits, depositScale)
	}

	voluntaryExits := []scale.VoluntaryExit{}

	for _, voluntaryExit := range body.VoluntaryExits {
		voluntaryExitScale, err := voluntaryExit.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		voluntaryExits = append(voluntaryExits, voluntaryExitScale)
	}

	depositCount, err := toUint64(body.Eth1Data.DepositCount)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	executionPayload := body.ExecutionPayload

	baseFeePerGasUint64, err := toUint64(executionPayload.BaseFeePerGas)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	bigInt := big.NewInt(int64(baseFeePerGasUint64))

	blockNumber, err := toUint64(executionPayload.BlockNumber)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	gasLimit, err := toUint64(executionPayload.GasLimit)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	gasUsed, err := toUint64(executionPayload.GasUsed)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	timestamp, err := toUint64(executionPayload.Timestamp)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	transactions, err := getTransactionsHashTreeRoot(executionPayload.Transactions)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	randaoReveal, err := hexStringToByteArray(body.RandaoReveal)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	feeRecipient, err := hexStringToByteArray(executionPayload.FeeRecipient)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	logsBloom, err := hexStringToByteArray(executionPayload.LogsBloom)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	extraData, err := hexStringToByteArray(executionPayload.ExtraData)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	return scale.BeaconBlock{
		Slot:          types.NewU64(slot),
		ProposerIndex: types.NewU64(proposerIndex),
		ParentRoot:    types.NewH256(common.HexToHash(dataMessage.ParentRoot).Bytes()),
		StateRoot:     types.NewH256(common.HexToHash(dataMessage.StateRoot).Bytes()),
		Body: scale.Body{
			RandaoReveal: randaoReveal,
			Eth1Data: scale.Eth1Data{
				DepositRoot:  types.NewH256(common.HexToHash(body.Eth1Data.DepositRoot).Bytes()),
				DepositCount: types.NewU64(depositCount),
				BlockHash:    types.NewH256(common.HexToHash(body.Eth1Data.BlockHash).Bytes()),
			},
			Graffiti:          types.NewH256(common.HexToHash(body.Graffiti).Bytes()),
			ProposerSlashings: proposerSlashings,
			AttesterSlashings: attesterSlashings,
			Attestations:      attestations,
			Deposits:          deposits,
			VoluntaryExits:    voluntaryExits,
			SyncAggregate:     syncAggregate,
			ExecutionPayload: scale.ExecutionPayload{
				ParentHash:    types.NewH256(common.HexToHash(executionPayload.ParentHash).Bytes()),
				FeeRecipient:  feeRecipient,
				StateRoot:     types.NewH256(common.HexToHash(executionPayload.StateRoot).Bytes()),
				ReceiptsRoot:  types.NewH256(common.HexToHash(executionPayload.ReceiptsRoot).Bytes()),
				LogsBloom:     logsBloom,
				PrevRandao:    types.NewH256(common.HexToHash(executionPayload.PrevRandao).Bytes()),
				BlockNumber:   types.NewU64(blockNumber),
				GasLimit:      types.NewU64(gasLimit),
				GasUsed:       types.NewU64(gasUsed),
				Timestamp:     types.NewU64(timestamp),
				ExtraData:     extraData,
				BaseFeePerGas: types.NewU256(*bigInt),
				BlockHash:     types.NewH256(common.HexToHash(executionPayload.BlockHash).Bytes()),
				Transactions:  transactions,
			},
		},
	}, nil
}

func (p ProposerSlashingResponse) ToScale() (scale.ProposerSlashing, error) {
	signedHeader1, err := p.SignedHeader1.ToScale()
	if err != nil {
		return scale.ProposerSlashing{}, err
	}

	signedHeader2, err := p.SignedHeader2.ToScale()
	if err != nil {
		return scale.ProposerSlashing{}, err
	}

	return scale.ProposerSlashing{
		SignedHeader1: signedHeader1,
		SignedHeader2: signedHeader2,
	}, nil
}

func (a AttesterSlashingResponse) ToScale() (scale.AttesterSlashing, error) {
	attestation1, err := a.Attestation1.ToScale()
	if err != nil {
		return scale.AttesterSlashing{}, err
	}

	attestation2, err := a.Attestation2.ToScale()
	if err != nil {
		return scale.AttesterSlashing{}, err
	}

	return scale.AttesterSlashing{
		Attestation1: attestation1,
		Attestation2: attestation2,
	}, nil
}

func (a AttestationResponse) ToScale() (scale.Attestation, error) {
	data, err := a.Data.ToScale()
	if err != nil {
		return scale.Attestation{}, err
	}

	aggregationBits, err := hexStringToByteArray(a.AggregationBits)
	if err != nil {
		return scale.Attestation{}, err
	}

	signature, err := hexStringToByteArray(a.Signature)
	if err != nil {
		return scale.Attestation{}, err
	}

	return scale.Attestation{
		AggregationBits: aggregationBits,
		Data:            data,
		Signature:       signature,
	}, nil
}

func (d VoluntaryExitResponse) ToScale() (scale.VoluntaryExit, error) {
	epoch, err := toUint64(d.Epoch)
	if err != nil {
		return scale.VoluntaryExit{}, err
	}

	validaterIndex, err := toUint64(d.ValidatorIndex)
	if err != nil {
		return scale.VoluntaryExit{}, err
	}

	return scale.VoluntaryExit{
		Epoch:          types.NewU64(epoch),
		ValidaterIndex: types.NewU64(validaterIndex),
	}, nil
}

func (d DepositResponse) ToScale() (scale.Deposit, error) {
	proofs := []types.H256{}

	for _, proofData := range d.Proof {
		proofs = append(proofs, types.NewH256(common.HexToHash(proofData).Bytes()))
	}

	amount, err := toUint64(d.Data.Amount)
	if err != nil {
		return scale.Deposit{}, err
	}

	pubkey, err := hexStringToByteArray(d.Data.Pubkey)
	if err != nil {
		return scale.Deposit{}, err
	}

	signature, err := hexStringToByteArray(d.Data.Signature)
	if err != nil {
		return scale.Deposit{}, err
	}

	return scale.Deposit{
		Proof: proofs,
		Data: scale.DepositData{
			Pubkey:                pubkey,
			WithdrawalCredentials: types.NewH256(common.HexToHash(d.Data.WithdrawalCredentials).Bytes()),
			Amount:                types.NewU64(amount),
			Signature:             signature,
		},
	}, nil
}

func (s SignedHeaderResponse) ToScale() (scale.SignedHeader, error) {
	message, err := s.Message.ToScale()
	if err != nil {
		return scale.SignedHeader{}, err
	}

	return scale.SignedHeader{
		Message:   message,
		Signature: s.Signature,
	}, nil
}

func (i IndexedAttestationResponse) ToScale() (scale.IndexedAttestation, error) {
	data, err := i.Data.ToScale()
	if err != nil {
		return scale.IndexedAttestation{}, err
	}

	attestationIndexes := []types.U64{}

	for _, index := range i.AttestingIndices {
		indexInt, err := toUint64(index)
		if err != nil {
			return scale.IndexedAttestation{}, err
		}

		attestationIndexes = append(attestationIndexes, types.NewU64(indexInt))
	}

	signature, err := hexStringToByteArray(i.Signature)
	if err != nil {
		return scale.IndexedAttestation{}, err
	}

	return scale.IndexedAttestation{
		AttestingIndices: attestationIndexes,
		Data:             data,
		Signature:        signature,
	}, nil
}

func (a AttestationDataResponse) ToScale() (scale.AttestationData, error) {
	slot, err := toUint64(a.Slot)
	if err != nil {
		return scale.AttestationData{}, err
	}

	index, err := toUint64(a.Index)
	if err != nil {
		return scale.AttestationData{}, err
	}

	source, err := a.Source.ToScale()
	if err != nil {
		return scale.AttestationData{}, err
	}

	target, err := a.Target.ToScale()
	if err != nil {
		return scale.AttestationData{}, err
	}

	return scale.AttestationData{
		Slot:            types.NewU64(slot),
		Index:           types.NewU64(index),
		BeaconBlockRoot: types.NewH256(common.HexToHash(a.BeaconBlockRoot).Bytes()),
		Source:          source,
		Target:          target,
	}, nil
}

func (c CheckpointResponse) ToScale() (scale.Checkpoint, error) {
	epoch, err := toUint64(c.Epoch)
	if err != nil {
		return scale.Checkpoint{}, err
	}

	return scale.Checkpoint{
		Epoch: types.NewU64(epoch),
		Root:  types.NewH256(common.HexToHash(c.Root).Bytes()),
	}, nil
}

func toUint64(stringVal string) (uint64, error) {
	intVal, err := strconv.ParseUint(stringVal, 10, 64)
	if err != nil {
		return 0, err
	}

	return intVal, err
}

func proofBranchToScale(proofs []common.Hash) []types.H256 {
	syncCommitteeBranch := []types.H256{}

	for _, proof := range proofs {
		syncCommitteeBranch = append(syncCommitteeBranch, types.NewH256(proof.Bytes()))
	}

	return syncCommitteeBranch
}

func getTransactionsHashTreeRoot(transactions []string) (types.H256, error) {
	resultTransactions := [][]byte{}

	for _, trans := range transactions {
		decodeString, err := hex.DecodeString(strings.ReplaceAll(trans, "0x", ""))
		if err != nil {
			return types.H256{}, err
		}
		resultTransactions = append(resultTransactions, decodeString)
	}

	hh := ssz.DefaultHasherPool.Get()

	indx := hh.Index()

	{
		subIndx := hh.Index()
		num := uint64(len(resultTransactions))
		if num > 1048576 {
			err := ssz.ErrIncorrectListSize
			return types.H256{}, err
		}
		for _, elem := range resultTransactions {
			{
				elemIndx := hh.Index()
				byteLen := uint64(len(elem))
				if byteLen > 1073741824 {
					err := ssz.ErrIncorrectListSize
					return types.H256{}, err
				}
				hh.AppendBytes32(elem)
				hh.MerkleizeWithMixin(elemIndx, byteLen, (1073741824+31)/32)
			}
		}
		hh.MerkleizeWithMixin(subIndx, num, 1048576)
	}

	hh.Merkleize(indx)

	root, err := hh.HashRoot()
	if err != nil {
		return types.H256{}, err
	}

	return types.NewH256(root[:]), nil
}
