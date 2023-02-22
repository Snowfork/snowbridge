package syncer

import (
	"encoding/hex"
	"errors"
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"math"
	"os"
	"strconv"
	"strings"
)

const BlockRootGeneralizedIndex = 37

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrBeaconStateAvailableYet                    = errors.New("beacon state object not available yet")
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
	Payload                  SyncCommitteePeriodPayload
	FinalizedHeaderBlockRoot common.Hash
	BlockRootsTree           *ssz.Node
}

type SyncCommitteePeriodPayload struct {
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
	Payload                  FinalizedHeaderPayload
	FinalizedHeaderBlockRoot common.Hash
	BlockRootsTree           *ssz.Node
}

type HeaderUpdate struct {
	Block                         scale.BeaconBlock
	SyncAggregate                 scale.SyncAggregate
	SignatureSlot                 types.U64
	BlockRootProof                []types.H256
	BlockRootProofFinalizedHeader types.H256
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

func (s *Syncer) GetInitialSync() (InitialSync, error) {
	checkpoint, err := s.Client.GetFinalizedCheckpoint()
	if err != nil {
		return InitialSync{}, fmt.Errorf("get finalized checkpoint: %w", err)
	}

	bootstrap, err := s.Client.GetBootstrap(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return InitialSync{}, fmt.Errorf("get bootstrap: %w", err)
	}

	genesis, err := s.Client.GetGenesis()
	if err != nil {
		return InitialSync{}, fmt.Errorf("get genesis: %w", err)
	}

	return InitialSync{
		Header:                     bootstrap.Header,
		CurrentSyncCommittee:       SyncCommitteeJSON(bootstrap.CurrentSyncCommittee),
		CurrentSyncCommitteeBranch: bootstrap.CurrentSyncCommitteeBranch,
		ValidatorsRoot:             genesis.ValidatorsRoot.Hex(),
		ImportTime:                 genesis.Time,
	}, nil
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

	log.WithFields(log.Fields{"finalizedHeader": common.HexToHash(finalizedHeader.StateRoot.Hex())}).Info("getting block root proof")

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

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("fetch block roots: %w", err)
	}

	finalizedHeaderBlockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return SyncCommitteePeriodUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	syncCommitteePeriodUpdate := SyncCommitteePeriodUpdate{
		Payload: SyncCommitteePeriodPayload{
			AttestedHeader:          attestedHeader,
			NextSyncCommittee:       nextSyncCommittee,
			NextSyncCommitteeBranch: proofBranchToScale(committeeUpdate.NextSyncCommitteeBranch),
			FinalizedHeader:         finalizedHeader,
			FinalityBranch:          proofBranchToScale(committeeUpdate.FinalityBranch),
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

func (s *Syncer) GetBlockRoots(slot uint64) (BlockRootProof, error) {
	log.Info("getting beacon state")
	err := s.Client.DownloadBeaconState(fmt.Sprintf("%d", slot))
	switch {
	case errors.Is(err, ErrNotFound):
		return BlockRootProof{}, ErrBeaconStateAvailableYet
	case err != nil:
		return BlockRootProof{}, err
	}

	log.Info("done getting beacon state")

	const fileName = "beacon_state.ssz"

	defer func() {
		_ = os.Remove(fileName)
	}()

	data, err := os.ReadFile(fileName)
	if err != nil {
		return BlockRootProof{}, fmt.Errorf("find beacon state file: %w", err)
	}

	log.Info("reading file")

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

	stateCheck := stateTree.Hash() // necessary to populate the proof tree values

	log.WithFields(log.Fields{"spec": s.activeSpec, "stateHash": common.BytesToHash(stateCheck)}).Info("getting block root proof")

	proof, err := stateTree.Prove(BlockRootGeneralizedIndex)
	if err != nil {
		log.WithError(err).Info("block proof error")

		return BlockRootProof{}, fmt.Errorf("get block roof proof: %w", err)
	}

	scaleBlockRootProof := []types.H256{}
	for _, proofItem := range proof.Hashes {
		scaleBlockRootProof = append(scaleBlockRootProof, types.NewH256(proofItem))
	}

	blockRootsContainer.SetBlockRoots(beaconState.GetBlockRoots())

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

	blockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return FinalizedHeaderUpdate{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
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
		Payload:                  finalizedHeaderUpdate,
		FinalizedHeaderBlockRoot: blockRoot,
	}, nil
}

func (s *Syncer) getNextBlockRootBySlot(slot uint64) (common.Hash, error) {
	err := ErrNotFound
	var blockRoot common.Hash
	tries := 0
	maxSlotsMissed := int(s.SlotsInEpoch)
	for errors.Is(err, ErrNotFound) && tries < maxSlotsMissed {
		blockRoot, err = s.Client.GetBeaconBlockRoot(slot)
		if err != nil && !errors.Is(err, ErrNotFound) {
			return blockRoot, fmt.Errorf("fetch block: %w", err)
		}

		if errors.Is(err, ErrNotFound) {
			log.WithField("slot", slot).Info("no block at slot")
			tries = tries + 1
			slot = slot + 1
		}
	}

	return blockRoot, nil
}

func (s *Syncer) GetNextHeaderUpdateBySlotWithAncestryProof(slot uint64, checkpoint cache.Proof) (HeaderUpdate, error) {
	blockRoot, err := s.getNextBlockRootBySlot(slot)
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("get next block root by slot: %w", err)
	}

	return s.GetHeaderUpdateWithAncestryProof(blockRoot, checkpoint)
}

func (s *Syncer) GetNextHeaderUpdateBySlot(slot uint64) (HeaderUpdate, error) {
	blockRoot, err := s.getNextBlockRootBySlot(slot)
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("get next block root by slot: %w", err)
	}

	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	blockScale, err := block.ToScale()
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("convert block to scale: %w", err)
	}

	headerUpdate := HeaderUpdate{
		Block: blockScale,
	}

	return headerUpdate, nil
}

func (s *Syncer) GetHeaderUpdateWithAncestryProof(blockRoot common.Hash, checkpoint cache.Proof) (HeaderUpdate, error) {
	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("fetch block: %w", err)
	}

	blockScale, err := block.ToScale()
	if err != nil {
		return HeaderUpdate{}, fmt.Errorf("convert block to scale: %w", err)
	}

	slot := uint64(blockScale.Slot)

	// If slot == finalizedSlot, there won't be an ancestry proof because the header state in question is also the
	// finalized header
	if slot == checkpoint.Slot {
		log.WithFields(log.Fields{"blockRoot": blockRoot, "slot": int(blockScale.Slot)}).Info("no ancestry proof available, finalized block")

		return HeaderUpdate{
			Block:          blockScale,
			BlockRootProof: []types.H256{},
		}, nil
	}

	proofScale, err := s.getBlockHeaderAncestryProof(int(blockScale.Slot), blockRoot, checkpoint.BlockRootsTree)

	displayProof := []common.Hash{}
	for _, proof := range proofScale {
		displayProof = append(displayProof, common.HexToHash(proof.Hex()))
	}

	log.WithFields(log.Fields{"checkpointSlot": checkpoint.Slot, "blockRoot": blockRoot, "slot": int(blockScale.Slot), "BlockRootProof": displayProof, "BlockRootProofFinalizedHeader": checkpoint.FinalizedBlockRoot}).Info("checkpoint slot used for proof")

	headerUpdate := HeaderUpdate{
		Block:                         blockScale,
		BlockRootProof:                proofScale,
		BlockRootProofFinalizedHeader: types.NewH256(checkpoint.FinalizedBlockRoot.Bytes()),
	}

	return headerUpdate, nil
}

func (s *Syncer) getBlockHeaderAncestryProof(slot int, blockRoot common.Hash, blockRootTree *ssz.Node) ([]types.H256, error) {
	indexInArray := slot % s.MaxSlotsPerHistoricalRoot

	treeDepth := math.Floor(math.Log2(float64(s.MaxSlotsPerHistoricalRoot)))
	leavesStartIndex := int(math.Pow(2, treeDepth))
	leafIndex := leavesStartIndex + indexInArray

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
