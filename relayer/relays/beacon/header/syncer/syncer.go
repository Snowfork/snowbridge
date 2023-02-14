package syncer

import (
	"encoding/hex"
	"errors"
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"math"
	"math/big"
	"os"
	"strconv"
	"strings"
)

const BlockRootGeneralizedIndex = 37

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrBeaconStateAvailableYet                    = errors.New("beacon state object not available yet")
	ErrFinalizedHeaderUnchanged                   = errors.New("finalized header unchanged")
)

type Syncer struct {
	Client                       BeaconClient
	SlotsInEpoch                 uint64
	EpochsPerSyncCommitteePeriod uint64
	MaxSlotsPerHistoricalRoot    int
	activeSpec                   config.ActiveSpec
}

func New(endpoint string, slotsInEpoch, epochsPerSyncCommitteePeriod uint64, maxSlotsPerHistoricalRoot int, activeSpec config.ActiveSpec) *Syncer {
	return &Syncer{
		Client:                       *NewBeaconClient(endpoint),
		SlotsInEpoch:                 slotsInEpoch,
		EpochsPerSyncCommitteePeriod: epochsPerSyncCommitteePeriod,
		MaxSlotsPerHistoricalRoot:    maxSlotsPerHistoricalRoot,
		activeSpec:                   activeSpec,
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

type BlockRootProof struct {
	Leaf  types.H256
	Proof []types.H256
	Tree  *ssz.Node
}

type SyncCommitteePeriodUpdate struct {
	AttestedHeader          scale.BeaconHeader
	NextSyncCommittee       scale.CurrentSyncCommittee
	NextSyncCommitteeBranch []types.H256
	FinalizedHeader         scale.BeaconHeader
	FinalityBranch          []types.H256
	SyncAggregate           scale.SyncAggregate
	SyncCommitteePeriod     types.U64
	SignatureSlot           types.U64
	BlockRootsHash          types.H256
	BlockRootProof          []types.H256
}

type FinalizedHeaderPayload struct {
	AttestedHeader  scale.BeaconHeader
	FinalizedHeader scale.BeaconHeader
	FinalityBranch  []types.H256
	SyncAggregate   scale.SyncAggregate
	SignatureSlot   types.U64
	BlockRootsHash  types.H256
	BlockRootProof  []types.H256
}

type FinalizedHeaderUpdate struct {
	Payload             FinalizedHeaderPayload
	FinalizedHeaderHash common.Hash
	BlockRootsTree      *ssz.Node
}

type HeaderUpdate struct {
	Block          scale.BeaconBlock
	SyncAggregate  scale.SyncAggregate
	SignatureSlot  types.U64
	BlockRootProof []types.H256
}

func (s *Syncer) GetSyncPeriodsToFetch(lastSyncedPeriod, currentSlot uint64) ([]uint64, error) {
	currentSyncPeriod := s.ComputeSyncPeriodAtSlot(currentSlot)

	//The current sync period's next sync committee should be synced too. So even
	// if the syncing is up-to-date with the current period, we still need to sync the current
	// period's next sync committee.
	if lastSyncedPeriod == currentSyncPeriod {
		return []uint64{currentSyncPeriod}, nil
	}

	syncPeriodsToFetch := []uint64{}
	for i := lastSyncedPeriod; i <= currentSyncPeriod; i++ {
		syncPeriodsToFetch = append(syncPeriodsToFetch, i)
	}

	return syncPeriodsToFetch, nil
}

func (s *Syncer) GetSyncCommitteePeriodUpdate(from uint64) (SyncCommitteePeriodUpdate, error) {
	committeeUpdateContainer, err := s.Client.GetSyncCommitteePeriodUpdate(from)
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch sync committee period update: %w", err)
	}

	committeeUpdate := committeeUpdateContainer.Data

	attestedHeader, err := committeeUpdate.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := committeeUpdate.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	nextSyncCommittee, err := committeeUpdate.NextSyncCommittee.ToScale()
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}

	syncAggregate, err := committeeUpdate.SyncAggregate.ToScale()
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(committeeUpdate.SignatureSlot, 10, 64)
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(committeeUpdate.FinalizedHeader.Beacon.StateRoot)
	switch {
	case errors.Is(err, ErrBeaconStateAvailableYet):
		log.WithFields(log.Fields{
			"stateRoot": committeeUpdate.FinalizedHeader.Beacon.StateRoot,
			"slot":      committeeUpdate.FinalizedHeader.Beacon.Slot,
		}).Info("sync committee update not available yet")
		//return SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	case err != nil:
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncCommitteePeriodUpdate := SyncCommitteePeriodUpdate{
		AttestedHeader:          attestedHeader,
		NextSyncCommittee:       nextSyncCommittee,
		NextSyncCommitteeBranch: proofBranchToScale(committeeUpdate.NextSyncCommitteeBranch),
		FinalizedHeader:         finalizedHeader,
		FinalityBranch:          proofBranchToScale(committeeUpdate.FinalityBranch),
		SyncAggregate:           syncAggregate,
		SignatureSlot:           types.U64(signatureSlot),
		BlockRootsHash:          blockRootsProof.Leaf,
		BlockRootProof:          blockRootsProof.Proof,
	}

	finalizedHeaderSlot := s.ComputeSyncPeriodAtSlot(uint64(finalizedHeader.Slot))

	if finalizedHeaderSlot != from {
		return syncCommitteePeriodUpdate, ErrCommitteeUpdateHeaderInDifferentSyncPeriod
	}

	return syncCommitteePeriodUpdate, nil
}

func (s *Syncer) GetBlockRoots(stateRoot string) (BlockRootProof, error) {
	err := s.Client.DownloadBeaconState(stateRoot)
	switch {
	case errors.Is(err, ErrNotFound):
		return BlockRootProof{}, ErrBeaconStateAvailableYet
	case err != nil:
		return BlockRootProof{}, err
	}

	const fileName = "beacon_state.ssz"

	defer func() {
		_ = os.Remove(fileName)
	}()

	data, err := os.ReadFile(fileName)
	if err != nil {
		return BlockRootProof{}, fmt.Errorf("find beacon state file: %w", err)
	}

	var beaconState state.BeaconState
	var blockRootsContainer state.BlockRootsContainer

	if s.activeSpec == config.Minimal {
		blockRootsContainer = &state.BlockRootsContainerMinimal{}
		beaconState = &state.BeaconStateBellatrixMinimal{}
	} else {
		blockRootsContainer = &state.BlockRootsContainerMainnet{}
		beaconState = &state.BeaconStateBellatrixMainnet{}
	}

	err = beaconState.UnmarshalSSZ(data)
	if err != nil {
		return BlockRootProof{}, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	stateTree, err := beaconState.GetTree()
	if err != nil {
		return BlockRootProof{}, fmt.Errorf("get state tree: %w", err)
	}

	checkStateRoot := stateTree.Hash()

	// todo remove
	if common.BytesToHash(checkStateRoot).Hex() != stateRoot {
		return BlockRootProof{}, fmt.Errorf("computed state hash tree root (%s) does not match known state root (%s)", common.BytesToHash(checkStateRoot).Hex(), stateRoot)
	}

	proof, err := stateTree.Prove(BlockRootGeneralizedIndex)
	if err != nil {
		log.WithError(err).Info("block proof error")

		return BlockRootProof{}, fmt.Errorf("get block roof proof: %w", err)
	}

	scaleBlockRootProof := []types.H256{}
	for _, proofItem := range proof.Hashes {
		scaleBlockRootProof = append(scaleBlockRootProof, types.NewH256(proofItem))
	}

	// todo remove sanity check
	ok, err := ssz.VerifyProof(common.FromHex(stateRoot), proof)
	if err != nil {
		log.WithError(err).Info("proof error")

		return BlockRootProof{}, fmt.Errorf("proof error: %w", err)
	}
	if !ok {
		return BlockRootProof{}, fmt.Errorf("proof failed")
	}

	blockRootsContainer.SetBlockRoots(beaconState.GetBlockRoots())

	displayBlockRoots := make(map[int]common.Hash)

	for i, blockRootAtIndex := range beaconState.GetBlockRoots() {
		displayBlockRoots[i] = common.BytesToHash(blockRootAtIndex)
	}

	tree, err := blockRootsContainer.GetTree()
	if err != nil {
		return BlockRootProof{}, fmt.Errorf("convert block roots to tree: %w", err)
	}

	return BlockRootProof{
		Leaf:  types.NewH256(proof.Leaf),
		Proof: scaleBlockRootProof,
		Tree:  tree,
	}, nil
}

func (s *Syncer) GetFinalizedUpdate() (FinalizedHeaderUpdate, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	finalizedHeaderSSZ := state.BeaconBlockHeader{
		Slot:          uint64(finalizedHeader.Slot),
		ProposerIndex: uint64(finalizedHeader.ProposerIndex),
		ParentRoot:    common.FromHex(finalizedHeader.ParentRoot.Hex()),
		StateRoot:     common.FromHex(finalizedHeader.StateRoot.Hex()),
		BodyRoot:      common.FromHex(finalizedHeader.BodyRoot.Hex()),
	}

	blockRoot, err := finalizedHeaderSSZ.HashTreeRoot()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	// TODO remove after tests
	checkBlockRoot, err := s.Client.GetBeaconBlockRoot(uint64(finalizedHeader.Slot))
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("fetch block root: %w", err)
	}

	if blockRoot != checkBlockRoot {
		return FinalizedHeaderUpdate{}, fmt.Errorf("expected block root does not match actual block root as retrieved from API: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(finalizedUpdate.Data.FinalizedHeader.Beacon.StateRoot)
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(finalizedUpdate.Data.SignatureSlot, 10, 64)
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	finalizedHeaderUpdate := FinalizedHeaderPayload{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  proofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		SignatureSlot:   types.U64(signatureSlot),
		BlockRootsHash:  blockRootsProof.Leaf,
		BlockRootProof:  blockRootsProof.Proof,
	}

	return FinalizedHeaderUpdate{
		Payload:             finalizedHeaderUpdate,
		FinalizedHeaderHash: blockRoot,
		BlockRootsTree:      blockRootsProof.Tree,
	}, nil
}

func (s *Syncer) GetLatestFinalizedHeader() (FinalizedHeaderUpdate, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	finalizedHeaderSSZ := state.BeaconBlockHeader{
		Slot:          uint64(finalizedHeader.Slot),
		ProposerIndex: uint64(finalizedHeader.ProposerIndex),
		ParentRoot:    common.FromHex(finalizedHeader.ParentRoot.Hex()),
		StateRoot:     common.FromHex(finalizedHeader.StateRoot.Hex()),
		BodyRoot:      common.FromHex(finalizedHeader.BodyRoot.Hex()),
	}

	blockRoot, err := finalizedHeaderSSZ.HashTreeRoot()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(finalizedUpdate.Data.SignatureSlot, 10, 64)
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	finalizedHeaderUpdate := FinalizedHeaderPayload{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  proofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		SignatureSlot:   types.U64(signatureSlot),
	}

	return FinalizedHeaderUpdate{
		Payload:             finalizedHeaderUpdate,
		FinalizedHeaderHash: blockRoot,
	}, nil
}

func (s *Syncer) GetNextHeaderUpdateBySlot(slot uint64, blockRootTree *ssz.Node, finalizedSlot uint64) (HeaderUpdate, error) {
	err := ErrNotFound
	var blockRoot common.Hash
	tries := 0
	maxSlotsMissed := int(s.SlotsInEpoch)
	for errors.Is(err, ErrNotFound) && tries < maxSlotsMissed {
		blockRoot, err = s.Client.GetBeaconBlockRoot(slot)
		if err != nil && !errors.Is(err, ErrNotFound) {
			return HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
		}

		if errors.Is(err, ErrNotFound) {
			log.WithField("slot", slot).Info("no block at slot")
			tries = tries + 1
			slot = slot + 1
		}
	}

	return s.GetHeaderUpdate(blockRoot, blockRootTree, finalizedSlot)
}

func (s *Syncer) GetHeaderUpdate(blockRoot common.Hash, blockRootTree *ssz.Node, finalizedSlot uint64) (HeaderUpdate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	blockScale, err := block.ToScale()
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("convert block to scale: %w", err)
	}

	log.WithFields(log.Fields{"blockRoot": blockRoot, "slot": int(blockScale.Slot)}).Info("block root at slot is")

	slot := uint64(blockScale.Slot)

	// If slot == finalizedSlot, there won't be an ancestry proof because the header state in question is also the
	// finalized header
	if slot == finalizedSlot {
		log.WithFields(log.Fields{"blockRoot": blockRoot, "slot": int(blockScale.Slot)}).Info("no ancestry proof available, finalized block")

		return HeaderUpdate{
			Block:          blockScale,
			BlockRootProof: []types.H256{},
		}, nil
	}

	proofScale, err := s.getBlockHeaderAncestryProof(int(blockScale.Slot), blockRoot, blockRootTree)

	headerUpdate := HeaderUpdate{
		Block:          blockScale,
		BlockRootProof: proofScale,
	}

	return headerUpdate, nil
}

func (s *Syncer) getBlockHeaderAncestryProof(slot int, blockRoot common.Hash, blockRootTree *ssz.Node) ([]types.H256, error) {
	indexInArray := slot % s.MaxSlotsPerHistoricalRoot

	treeDepth := math.Floor(math.Log2(float64(s.MaxSlotsPerHistoricalRoot)))
	leavesStartIndex := int(math.Pow(2, treeDepth))
	leafIndex := leavesStartIndex + indexInArray

	log.WithFields(log.Fields{
		"treeDepth":                 treeDepth,
		"leavesStartIndex":          leavesStartIndex,
		"indexInArray":              indexInArray,
		"leafIndex":                 leafIndex,
		"slot":                      slot,
		"maxSlotsPerHistoricalRoot": s.MaxSlotsPerHistoricalRoot,
	}).Info("blockHashAtIndex")

	proof, err := blockRootTree.Prove(leafIndex)
	if err != nil {
		return nil, fmt.Errorf("get block proof: %w", err)
	}

	if common.BytesToHash(proof.Leaf) != blockRoot {
		return nil, fmt.Errorf("block root at index (%s) does not match expected block root (%s)", common.BytesToHash(proof.Leaf), blockRoot)
	}

	// sanity check
	//ok, err := ssz.VerifyProof(s.currentFinalizedHeader.blockRootProofHash[:], proof)
	//if err != nil {
	//	return nil, fmt.Errorf("block proof at index errored: %w", err)
	//}
	//if ok != true {
	//	return nil, fmt.Errorf("block proof at index failed")
	//}

	proofScale := []types.H256{}
	for _, proofItem := range proof.Hashes {
		proofScale = append(proofScale, types.NewH256(proofItem))
	}

	return proofScale, nil
}

func (s *Syncer) GetSyncAggregate(blockRoot common.Hash) (scale.SyncAggregate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return scale.SyncAggregate{}, fmt.Errorf("fetch block: %w", err)
	}

	blockScale, err := block.ToScale()
	if err != nil {
		return scale.SyncAggregate{}, fmt.Errorf("convert block to scale: %w", err)
	}

	return blockScale.Body.SyncAggregate, nil
}

func (s *Syncer) GetSyncAggregateForSlot(slot uint64) (scale.SyncAggregate, types.U64, error) {
	err := ErrNotFound
	var block BeaconBlockResponse
	tries := 0
	maxSlotsMissed := int(s.SlotsInEpoch)
	for errors.Is(err, ErrNotFound) && tries < maxSlotsMissed {
		log.WithFields(log.Fields{
			"try_number": tries,
			"slot":       slot,
		}).Info("fetching sync aggregate for slot")
		block, err = s.Client.GetBeaconBlockBySlot(slot)
		if err != nil && !errors.Is(err, ErrNotFound) {
			return scale.SyncAggregate{}, 0, fmt.Errorf("fetch block: %w", err)
		}

		tries = tries + 1
		slot = slot + 1
	}

	blockScale, err := block.ToScale()
	if err != nil {
		return scale.SyncAggregate{}, 0, fmt.Errorf("convert block to scale: %w", err)
	}
	return blockScale.Body.SyncAggregate, blockScale.Slot, nil
}

func (s *Syncer) ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (s.SlotsInEpoch * s.EpochsPerSyncCommitteePeriod)
}

func (s *Syncer) ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / s.SlotsInEpoch
}

func (s *Syncer) IsStartOfEpoch(slot uint64) bool {
	return slot%s.SlotsInEpoch == 0
}

func IsInArray(values []uint64, toCheck uint64) bool {
	for _, value := range values {
		if value == toCheck {
			return true
		}
	}
	return false
}

func IsInHashArray(values []common.Hash, toCheck common.Hash) bool {
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
	bytes, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return []byte{}, err
	}

	return bytes, nil
}

func (h HeaderResponse) ToScale() (scale.BeaconHeader, error) {
	slot, err := strconv.ParseUint(h.Slot, 10, 64)
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := strconv.ParseUint(h.ProposerIndex, 10, 64)
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("parse proposerIndex as int: %w", err)
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
			return scale.CurrentSyncCommittee{}, fmt.Errorf("convert sync committee pubkey to byte array: %w", err)
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := hexStringToPublicKey(s.AggregatePubkey)
	if err != nil {
		return scale.CurrentSyncCommittee{}, fmt.Errorf("convert sync committee aggregate bukey to byte array: %w", err)
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
		return scale.BeaconBlock{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := toUint64(dataMessage.ProposerIndex)
	if err != nil {
		return scale.BeaconBlock{}, fmt.Errorf("parse proposerIndex as int: %w", err)
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
