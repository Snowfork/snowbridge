package syncer

import (
	"errors"
	"fmt"
	"os"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/util"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

const (
	BlockRootGeneralizedIndex        = 37
	ExecutionPayloadGeneralizedIndex = 25
)

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrBeaconStateAvailableYet                    = errors.New("beacon state object not available yet")
)

type Syncer struct {
	Client                       api.BeaconClient
	SlotsInEpoch                 uint64
	EpochsPerSyncCommitteePeriod uint64
	MaxSlotsPerHistoricalRoot    int
	BlockRootIndexProofDepth     int
	activeSpec                   config.ActiveSpec
}

func New(endpoint string, slotsInEpoch, epochsPerSyncCommitteePeriod uint64, maxSlotsPerHistoricalRoot int, activeSpec config.ActiveSpec) *Syncer {
	return &Syncer{
		Client:                       *api.NewBeaconClient(endpoint, activeSpec, slotsInEpoch),
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

func (s *Syncer) GetCheckPoint() (scale.CheckPoint, error) {
	checkpoint, err := s.Client.GetFinalizedCheckpoint()
	if err != nil {
		return scale.CheckPoint{}, fmt.Errorf("get finalized checkpoint: %w", err)
	}

	bootstrap, err := s.Client.GetBootstrap(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return scale.CheckPoint{}, fmt.Errorf("get bootstrap: %w", err)
	}

	genesis, err := s.Client.GetGenesis()
	if err != nil {
		return scale.CheckPoint{}, fmt.Errorf("get genesis: %w", err)
	}

	header, err := bootstrap.Data.Header.Beacon.ToScale()
	if err != nil {
		return scale.CheckPoint{}, fmt.Errorf("convert header to scale: %w", err)
	}

	syncCommittee, err := bootstrap.Data.CurrentSyncCommittee.ToScale()
	if err != nil {
		return scale.CheckPoint{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}
	secondPerSlot := 6
	if s.activeSpec.IsMainnet() {
		secondPerSlot = 12
	}
	importTime := genesis.Time + uint64(header.Slot)*uint64(secondPerSlot)

	return scale.CheckPoint{
		Header:                     header,
		CurrentSyncCommittee:       syncCommittee,
		CurrentSyncCommitteeBranch: util.ProofBranchToScale(bootstrap.Data.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             types.H256(genesis.ValidatorsRoot),
		ImportTime:                 types.U64(importTime),
	}, nil
}

func (s *Syncer) GetSyncCommitteePeriodUpdate(from uint64) (scale.SyncCommitteePeriodUpdate, error) {
	committeeUpdateContainer, err := s.Client.GetSyncCommitteePeriodUpdate(from)
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch sync committee period update: %w", err)
	}

	committeeUpdate := committeeUpdateContainer.Data

	attestedHeader, err := committeeUpdate.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := committeeUpdate.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	nextSyncCommittee, err := committeeUpdate.NextSyncCommittee.ToScale()
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}

	syncAggregate, err := committeeUpdate.SyncAggregate.ToScale()
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(committeeUpdate.SignatureSlot, 10, 64)
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	}

	finalizedHeaderBlockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return scale.SyncCommitteePeriodUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	syncCommitteePeriodUpdate := scale.SyncCommitteePeriodUpdate{
		Payload: scale.SyncCommitteePeriodPayload{
			AttestedHeader:          attestedHeader,
			NextSyncCommittee:       nextSyncCommittee,
			NextSyncCommitteeBranch: util.ProofBranchToScale(committeeUpdate.NextSyncCommitteeBranch),
			FinalizedHeader:         finalizedHeader,
			FinalityBranch:          util.ProofBranchToScale(committeeUpdate.FinalityBranch),
			SyncAggregate:           syncAggregate,
			SignatureSlot:           types.U64(signatureSlot),
			BlockRootsHash:          blockRootsProof.Leaf,
			BlockRootProof:          blockRootsProof.Proof,
			SyncCommitteePeriod:     types.U64(from),
		},
		FinalizedHeaderBlockRoot: finalizedHeaderBlockRoot,
		BlockRootsTree:           blockRootsProof.Tree,
	}

	finalizedHeaderSlot := s.ComputeSyncPeriodAtSlot(uint64(finalizedHeader.Slot))

	if finalizedHeaderSlot != from {
		return syncCommitteePeriodUpdate, ErrCommitteeUpdateHeaderInDifferentSyncPeriod
	}

	return syncCommitteePeriodUpdate, nil
}

func (s *Syncer) GetBlockRoots(slot uint64) (scale.BlockRootProof, error) {
	beaconStateFilename, err := s.Client.DownloadBeaconState(fmt.Sprintf("%d", slot))
	switch {
	case errors.Is(err, api.ErrNotFound):
		return scale.BlockRootProof{}, ErrBeaconStateAvailableYet
	case err != nil:
		return scale.BlockRootProof{}, err
	}

	defer func() {
		_ = os.Remove(beaconStateFilename)
	}()

	data, err := os.ReadFile(beaconStateFilename)
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("find beacon state file: %w", err)
	}

	var beaconState state.BeaconState
	var blockRootsContainer state.BlockRootsContainer

	if s.activeSpec == config.Minimal {
		blockRootsContainer = &state.BlockRootsContainerMinimal{}
		beaconState = &state.BeaconStateCapellaMinimal{}
	} else {
		blockRootsContainer = &state.BlockRootsContainerMainnet{}
		beaconState = &state.BeaconStateCapellaMainnet{}
	}

	err = beaconState.UnmarshalSSZ(data)
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	stateTree, err := beaconState.GetTree()
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("get state tree: %w", err)
	}

	_ = stateTree.Hash() // necessary to populate the proof tree values

	proof, err := stateTree.Prove(BlockRootGeneralizedIndex)
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("get block roof proof: %w", err)
	}

	scaleBlockRootProof := []types.H256{}
	for _, proofItem := range proof.Hashes {
		scaleBlockRootProof = append(scaleBlockRootProof, types.NewH256(proofItem))
	}

	blockRootsContainer.SetBlockRoots(beaconState.GetBlockRoots())

	tree, err := blockRootsContainer.GetTree()
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("convert block roots to tree: %w", err)
	}

	return scale.BlockRootProof{
		Leaf:  types.NewH256(proof.Leaf),
		Proof: scaleBlockRootProof,
		Tree:  tree,
	}, nil
}

func (s *Syncer) GetFinalizedUpdate() (scale.FinalizedHeaderUpdate, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	blockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(finalizedUpdate.Data.SignatureSlot, 10, 64)
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	finalizedHeaderUpdate := scale.FinalizedHeaderPayload{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  util.ProofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		SignatureSlot:   types.U64(signatureSlot),
		BlockRootsHash:  blockRootsProof.Leaf,
		BlockRootProof:  blockRootsProof.Proof,
	}

	return scale.FinalizedHeaderUpdate{
		Payload:                  finalizedHeaderUpdate,
		FinalizedHeaderBlockRoot: blockRoot,
		BlockRootsTree:           blockRootsProof.Tree,
	}, nil
}

func (s *Syncer) HasFinalizedHeaderChanged(lastFinalizedBlockRoot common.Hash) (bool, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return false, fmt.Errorf("fetch finalized update: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return false, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	blockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return false, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	isTheSame := common.BytesToHash(blockRoot[:]).Hex() != lastFinalizedBlockRoot.Hex()

	return isTheSame, nil
}

func (s *Syncer) GetLatestFinalizedHeader() (scale.FinalizedHeaderUpdate, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert finalized header to scale: %w", err)
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
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(finalizedUpdate.Data.SignatureSlot, 10, 64)
	if err != nil {
		return scale.FinalizedHeaderUpdate{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	finalizedHeaderUpdate := scale.FinalizedHeaderPayload{
		AttestedHeader:  attestedHeader,
		FinalizedHeader: finalizedHeader,
		FinalityBranch:  util.ProofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		SyncAggregate:   syncAggregate,
		SignatureSlot:   types.U64(signatureSlot),
	}

	return scale.FinalizedHeaderUpdate{
		Payload:                  finalizedHeaderUpdate,
		FinalizedHeaderBlockRoot: blockRoot,
	}, nil
}

func (s *Syncer) getNextBlockRootBySlot(slot uint64) (common.Hash, error) {
	err := api.ErrNotFound
	var header api.BeaconHeader
	tries := 0
	maxSlotsMissed := int(s.SlotsInEpoch)
	for errors.Is(err, api.ErrNotFound) && tries < maxSlotsMissed {
		// Need to use GetHeaderBySlot instead of GetBeaconBlockRoot here because GetBeaconBlockRoot
		// returns the previous slot's block root if there is no block at the given slot
		header, err = s.Client.GetHeaderBySlot(slot)
		if err != nil && !errors.Is(err, api.ErrNotFound) {
			return common.Hash{}, fmt.Errorf("fetch block: %w", err)
		}

		if errors.Is(err, api.ErrNotFound) {
			log.WithField("slot", slot).Info("no block at slot")
			tries = tries + 1
			slot = slot + 1
		}
	}

	beaconHeader := state.BeaconBlockHeader{
		Slot:          header.Slot,
		ProposerIndex: header.ProposerIndex,
		ParentRoot:    header.ParentRoot.Bytes(),
		StateRoot:     header.StateRoot.Bytes(),
		BodyRoot:      header.BodyRoot.Bytes(),
	}

	computedRoot, err := beaconHeader.HashTreeRoot()
	if err != nil {
		return [32]byte{}, err
	}

	blockRoot, err := s.Client.GetBeaconBlockRoot(header.Slot)
	if err != nil && !errors.Is(err, api.ErrNotFound) {
		return blockRoot, fmt.Errorf("fetch block: %w", err)
	}

	log.WithFields(log.Fields{
		"computedRoot": common.BytesToHash(computedRoot[:]),
		"blockRoot":    blockRoot,
	}).Info("block roots")

	return blockRoot, nil
}

func (s *Syncer) GetNextHeaderUpdateBySlotWithAncestryProof(slot uint64, checkpoint cache.Proof) (scale.HeaderUpdate, error) {
	blockRoot, err := s.getNextBlockRootBySlot(slot)
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("get next block root by slot: %w", err)
	}

	return s.GetHeaderUpdateWithAncestryProof(blockRoot, checkpoint)
}

func (s *Syncer) GetNextHeaderUpdateBySlot(slot uint64) (scale.HeaderUpdate, error) {
	blockRoot, err := s.getNextBlockRootBySlot(slot)
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("get next block root by slot: %w", err)
	}

	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	header, err := s.Client.GetHeaderBySlot(block.GetBeaconSlot())
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	beaconHeader, err := header.ToScale()
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("beacon header to scale: %w", err)
	}

	executionPayloadScale, err := api.CapellaExecutionPayloadToScale(block.GetExecutionPayload(), s.activeSpec)
	if err != nil {
		return scale.HeaderUpdate{}, err
	}

	nextSyncCommittee := api.SyncAggregateToScale(block.GetSyncAggregate())

	executionHeaderBranch, err := s.getExecutionHeaderBranch(block)
	if err != nil {
		return scale.HeaderUpdate{}, err
	}

	headerUpdate := scale.HeaderUpdate{
		Payload: scale.HeaderUpdatePayload{
			BeaconHeader:    beaconHeader,
			ExecutionHeader: executionPayloadScale,
			ExecutionBranch: executionHeaderBranch,
		},
		NextSyncAggregate: nextSyncCommittee,
	}

	return headerUpdate, nil
}

func (s *Syncer) GetHeaderUpdateWithAncestryProof(blockRoot common.Hash, checkpoint cache.Proof) (scale.HeaderUpdate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	header, err := s.Client.GetHeaderBySlot(block.GetBeaconSlot())
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	beaconHeader, err := header.ToScale()
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("beacon header to scale: %w", err)
	}

	executionPayloadScale, err := api.CapellaExecutionPayloadToScale(block.GetExecutionPayload(), s.activeSpec)
	if err != nil {
		return scale.HeaderUpdate{}, err
	}

	nextSyncCommittee := api.SyncAggregateToScale(block.GetSyncAggregate())

	executionHeaderBranch, err := s.getExecutionHeaderBranch(block)
	if err != nil {
		return scale.HeaderUpdate{}, err
	}

	// If slot == finalizedSlot, there won't be an ancestry proof because the header state in question is also the
	// finalized header
	if block.GetBeaconSlot() == checkpoint.Slot {
		return scale.HeaderUpdate{
			Payload: scale.HeaderUpdatePayload{
				BeaconHeader:    beaconHeader,
				ExecutionHeader: executionPayloadScale,
				ExecutionBranch: executionHeaderBranch,
				BlockRootBranch: []types.H256{},
			},
			NextSyncAggregate: nextSyncCommittee,
		}, nil
	}

	proofScale, err := s.getBlockHeaderAncestryProof(int(block.GetBeaconSlot()), blockRoot, checkpoint.BlockRootsTree)
	if err != nil {
		return scale.HeaderUpdate{}, err
	}

	displayProof := []common.Hash{}
	for _, proof := range proofScale {
		displayProof = append(displayProof, common.HexToHash(proof.Hex()))
	}

	headerUpdate := scale.HeaderUpdate{
		Payload: scale.HeaderUpdatePayload{
			BeaconHeader:              beaconHeader,
			ExecutionHeader:           executionPayloadScale,
			ExecutionBranch:           executionHeaderBranch,
			BlockRootBranch:           proofScale,
			BlockRootBranchHeaderRoot: types.NewH256(checkpoint.FinalizedBlockRoot.Bytes()),
		},
		NextSyncAggregate: nextSyncCommittee,
	}

	return headerUpdate, nil
}

func (s *Syncer) getBlockHeaderAncestryProof(slot int, blockRoot common.Hash, blockRootTree *ssz.Node) ([]types.H256, error) {
	indexInArray := slot % s.MaxSlotsPerHistoricalRoot
	leafIndex := s.MaxSlotsPerHistoricalRoot + indexInArray

	if blockRootTree == nil {
		return nil, fmt.Errorf("block root tree is nil")
	}

	proof, err := blockRootTree.Prove(leafIndex)
	if err != nil {
		return nil, fmt.Errorf("get block proof: %w", err)
	}

	if common.BytesToHash(proof.Leaf) != blockRoot {
		return nil, fmt.Errorf("block root at index (%s) does not match expected block root (%s)", common.BytesToHash(proof.Leaf), blockRoot)
	}

	return util.BytesBranchToScale(proof.Hashes), nil
}

func (s *Syncer) getExecutionHeaderBranch(block state.BeaconBlock) ([]types.H256, error) {
	tree, err := block.GetBlockBodyTree()
	if err != nil {
		return nil, err
	}

	tree.Hash()

	proof, err := tree.Prove(ExecutionPayloadGeneralizedIndex)

	return util.BytesBranchToScale(proof.Hashes), nil
}

func (s *Syncer) GetSyncAggregate(blockRoot common.Hash) (scale.SyncAggregate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return scale.SyncAggregate{}, fmt.Errorf("fetch block: %w", err)
	}

	return api.SyncAggregateToScale(block.GetSyncAggregate()), nil
}

func (s *Syncer) GetSyncAggregateForSlot(slot uint64) (scale.SyncAggregate, types.U64, error) {
	err := api.ErrNotFound
	var block api.BeaconBlockResponse
	tries := 0
	maxSlotsMissed := int(s.SlotsInEpoch)
	for errors.Is(err, api.ErrNotFound) && tries < maxSlotsMissed {
		log.WithFields(log.Fields{
			"try_number": tries,
			"slot":       slot,
		}).Info("fetching sync aggregate for slot")
		block, err = s.Client.GetBeaconBlockBySlot(slot)
		if err != nil && !errors.Is(err, api.ErrNotFound) {
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
