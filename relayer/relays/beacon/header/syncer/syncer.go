package syncer

import (
	"errors"
	"fmt"
	"strconv"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"github.com/snowfork/snowbridge/relayer/relays/util"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
)

const (
	BlockRootGeneralizedIndex           = 37
	FinalizedCheckpointGeneralizedIndex = 105
	NextSyncCommitteeGeneralizedIndex   = 55
	ExecutionPayloadGeneralizedIndex    = 25
)

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrBeaconStateAvailableYet                    = errors.New("beacon state object not available yet")
)

type Syncer struct {
	Client   api.BeaconAPI
	store    store.BeaconStore
	protocol *protocol.Protocol
}

func New(client api.BeaconAPI, store store.BeaconStore, protocol *protocol.Protocol) *Syncer {
	return &Syncer{
		Client:   client,
		store:    store,
		protocol: protocol,
	}
}

type finalizedUpdateContainer struct {
	AttestedSlot        uint64
	AttestedState       state.BeaconState
	FinalizedState      state.BeaconState
	FinalizedHeader     api.BeaconHeader
	FinalizedCheckPoint state.Checkpoint
}

func (s *Syncer) GetCheckpoint() (scale.BeaconCheckpoint, error) {
	checkpoint, err := s.Client.GetFinalizedCheckpoint()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get finalized checkpoint: %w", err)
	}

	bootstrap, err := s.Client.GetBootstrap(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get bootstrap: %w", err)
	}

	genesis, err := s.Client.GetGenesis()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get genesis: %w", err)
	}

	header, err := bootstrap.Data.Header.Beacon.ToScale()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("convert header to scale: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(header.Slot))
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncCommittee, err := bootstrap.Data.CurrentSyncCommittee.ToScale()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}

	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get sync committee: %w", err)
	}

	return scale.BeaconCheckpoint{
		Header:                     header,
		CurrentSyncCommittee:       syncCommittee,
		CurrentSyncCommitteeBranch: util.ProofBranchToScale(bootstrap.Data.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             types.H256(genesis.ValidatorsRoot),
		BlockRootsRoot:             blockRootsProof.Leaf,
		BlockRootsBranch:           blockRootsProof.Proof,
	}, nil
}

// GetSyncCommitteePeriodUpdate fetches a sync committee update from the light client API endpoint. If it fails
// (typically because it cannot download the finalized header beacon state because the slot does not fall on a 32
// slot interval, due to a missed block), it will construct an update manually from data download from the beacon
// API, or if that is unavailable, use a stored beacon state.
func (s *Syncer) GetSyncCommitteePeriodUpdate(period uint64, lastFinalizedSlot uint64) (scale.Update, error) {
	update, err := s.GetSyncCommitteePeriodUpdateFromEndpoint(period)
	if err != nil {
		log.WithFields(log.Fields{"period": period, "err": err}).Warn("fetch sync committee update period light client failed, trying building update manually")
		update, err = s.GetFinalizedUpdateWithSyncCommittee(period, lastFinalizedSlot)
		if err != nil {
			return update, fmt.Errorf("build sync committee update: %w", err)
		}
	}

	return update, nil
}

// GetSyncCommitteePeriodUpdateFromEndpoint fetches a sync committee update from the light client API endpoint. If
// it cannot download the required beacon state from the API, it will look in the data store if the state is stored.
// If not, it returns an error.
func (s *Syncer) GetSyncCommitteePeriodUpdateFromEndpoint(from uint64) (scale.Update, error) {
	committeeUpdateContainer, err := s.Client.GetSyncCommitteePeriodUpdate(from)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch sync committee period update: %w", err)
	}

	committeeUpdate := committeeUpdateContainer.Data

	attestedHeader, err := committeeUpdate.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := committeeUpdate.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	nextSyncCommittee, err := committeeUpdate.NextSyncCommittee.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}

	syncAggregate, err := committeeUpdate.SyncAggregate.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(committeeUpdate.SignatureSlot, 10, 64)
	if err != nil {
		return scale.Update{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
	if err != nil {
		beaconStateData, err := s.store.GetBeaconStateData(uint64(finalizedHeader.Slot))
		if err != nil {
			return scale.Update{}, fmt.Errorf("fetch beacon state for block roots proof: %w", err)
		}
		beaconState, err := s.unmarshalBeaconState(uint64(finalizedHeader.Slot), beaconStateData)

		blockRootsProof, err = s.GetBlockRootsFromState(beaconState)
		if err != nil {
			return scale.Update{}, fmt.Errorf("fetch block roots: %w", err)
		}
	}

	finalizedHeaderBlockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return scale.Update{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	syncCommitteePeriodUpdate := scale.Update{
		Payload: scale.UpdatePayload{
			AttestedHeader: attestedHeader,
			SyncAggregate:  syncAggregate,
			SignatureSlot:  types.U64(signatureSlot),
			NextSyncCommitteeUpdate: scale.OptionNextSyncCommitteeUpdatePayload{
				HasValue: true,
				Value: scale.NextSyncCommitteeUpdatePayload{
					NextSyncCommittee:       nextSyncCommittee,
					NextSyncCommitteeBranch: util.ProofBranchToScale(committeeUpdate.NextSyncCommitteeBranch),
				},
			},
			FinalizedHeader:  finalizedHeader,
			FinalityBranch:   util.ProofBranchToScale(committeeUpdate.FinalityBranch),
			BlockRootsRoot:   blockRootsProof.Leaf,
			BlockRootsBranch: blockRootsProof.Proof,
		},
		FinalizedHeaderBlockRoot: finalizedHeaderBlockRoot,
		BlockRootsTree:           blockRootsProof.Tree,
	}

	finalizedPeriod := s.protocol.ComputeSyncPeriodAtSlot(uint64(finalizedHeader.Slot))

	if finalizedPeriod != from {
		return syncCommitteePeriodUpdate, ErrCommitteeUpdateHeaderInDifferentSyncPeriod
	}

	return syncCommitteePeriodUpdate, nil
}

func (s *Syncer) GetBlockRoots(slot uint64) (scale.BlockRootProof, error) {
	var blockRootProof scale.BlockRootProof
	var beaconState state.BeaconState
	var blockRootsContainer state.BlockRootsContainer

	data, err := s.Client.GetBeaconState(strconv.FormatUint(slot, 10))
	if err != nil {
		return blockRootProof, fmt.Errorf("download beacon state (at slot %d) failed: %w", slot, err)
	}
	isDeneb := s.protocol.DenebForked(slot)

	blockRootsContainer = &state.BlockRootsContainerMainnet{}
	if isDeneb {
		beaconState = &state.BeaconStateDenebMainnet{}
	} else {
		beaconState = &state.BeaconStateCapellaMainnet{}
	}

	err = beaconState.UnmarshalSSZ(data)
	if err != nil {
		return blockRootProof, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	stateTree, err := beaconState.GetTree()
	if err != nil {
		return blockRootProof, fmt.Errorf("get state tree: %w", err)
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
		return blockRootProof, fmt.Errorf("convert block roots to tree: %w", err)
	}

	return scale.BlockRootProof{
		Leaf:  types.NewH256(proof.Leaf),
		Proof: scaleBlockRootProof,
		Tree:  tree,
	}, nil
}

func (s *Syncer) GetBlockRootsFromState(beaconState state.BeaconState) (scale.BlockRootProof, error) {
	var blockRootProof scale.BlockRootProof
	var blockRootsContainer state.BlockRootsContainer

	blockRootsContainer = &state.BlockRootsContainerMainnet{}

	stateTree, err := beaconState.GetTree()
	if err != nil {
		return blockRootProof, fmt.Errorf("get state tree: %w", err)
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
		return blockRootProof, fmt.Errorf("convert block roots to tree: %w", err)
	}

	return scale.BlockRootProof{
		Leaf:  types.NewH256(proof.Leaf),
		Proof: scaleBlockRootProof,
		Tree:  tree,
	}, nil
}

func (s *Syncer) GetFinalizedUpdate() (scale.Update, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	attestedHeader, err := finalizedUpdate.Data.AttestedHeader.Beacon.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert attested header to scale: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	blockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return scale.Update{}, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(uint64(finalizedHeader.Slot))
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncAggregate, err := finalizedUpdate.Data.SyncAggregate.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	signatureSlot, err := strconv.ParseUint(finalizedUpdate.Data.SignatureSlot, 10, 64)
	if err != nil {
		return scale.Update{}, fmt.Errorf("parse signature slot as int: %w", err)
	}

	updatePayload := scale.UpdatePayload{
		AttestedHeader: attestedHeader,
		SyncAggregate:  syncAggregate,
		SignatureSlot:  types.U64(signatureSlot),
		NextSyncCommitteeUpdate: scale.OptionNextSyncCommitteeUpdatePayload{
			HasValue: false,
		},
		FinalizedHeader:  finalizedHeader,
		FinalityBranch:   util.ProofBranchToScale(finalizedUpdate.Data.FinalityBranch),
		BlockRootsRoot:   blockRootsProof.Leaf,
		BlockRootsBranch: blockRootsProof.Proof,
	}

	return scale.Update{
		Payload:                  updatePayload,
		FinalizedHeaderBlockRoot: blockRoot,
		BlockRootsTree:           blockRootsProof.Tree,
	}, nil
}

func (s *Syncer) HasFinalizedHeaderChanged(finalizedHeader scale.BeaconHeader, lastFinalizedBlockRoot common.Hash) (bool, error) {
	blockRoot, err := finalizedHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return false, fmt.Errorf("beacon header hash tree root: %w", err)
	}

	isTheSame := common.BytesToHash(blockRoot[:]).Hex() != lastFinalizedBlockRoot.Hex()

	return isTheSame, nil
}

func (s *Syncer) FindBeaconHeaderWithBlockIncluded(slot uint64) (state.BeaconBlockHeader, error) {
	err := api.ErrNotFound
	var header api.BeaconHeader
	tries := 0
	maxSlotsMissed := int(s.protocol.Settings.SlotsInEpoch)
	startSlot := slot
	for errors.Is(err, api.ErrNotFound) && tries < maxSlotsMissed {
		// Need to use GetHeaderBySlot instead of GetBeaconBlockRoot here because GetBeaconBlockRoot
		// returns the previous slot's block root if there is no block at the given slot
		header, err = s.Client.GetHeaderBySlot(slot)
		if err != nil && !errors.Is(err, api.ErrNotFound) {
			return state.BeaconBlockHeader{}, fmt.Errorf("fetch block: %w", err)
		}

		if errors.Is(err, api.ErrNotFound) {
			log.WithField("slot", slot).Info("skipped block not included")
			tries = tries + 1
			slot = slot + 1
		}
	}

	if err != nil || header.Slot == 0 {
		log.WithFields(logrus.Fields{
			"start": startSlot,
			"end":   slot,
		}).WithError(err).Error("matching block included not found")
		return state.BeaconBlockHeader{}, api.ErrNotFound
	}

	beaconHeader := state.BeaconBlockHeader{
		Slot:          header.Slot,
		ProposerIndex: header.ProposerIndex,
		ParentRoot:    header.ParentRoot.Bytes(),
		StateRoot:     header.StateRoot.Bytes(),
		BodyRoot:      header.BodyRoot.Bytes(),
	}

	return beaconHeader, nil
}

func (s *Syncer) GetHeaderUpdateBySlotWithCheckpoint(slot uint64, checkpoint *cache.Proof) (scale.HeaderUpdatePayload, error) {
	header, err := s.FindBeaconHeaderWithBlockIncluded(slot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("get next beacon header with block included: %w", err)
	}
	blockRoot, err := header.HashTreeRoot()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("header hash tree root: %w", err)
	}
	return s.GetHeaderUpdate(blockRoot, checkpoint)
}

func (s *Syncer) GetHeaderUpdate(blockRoot common.Hash, checkpoint *cache.Proof) (scale.HeaderUpdatePayload, error) {
	var update scale.HeaderUpdatePayload
	blockResponse, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return update, fmt.Errorf("fetch block: %w", err)
	}
	data := blockResponse.Data.Message
	slot, err := util.ToUint64(data.Slot)
	if err != nil {
		return update, err
	}

	sszBlock, err := blockResponse.ToFastSSZ(s.protocol.DenebForked(slot))
	if err != nil {
		return update, err
	}

	header, err := s.Client.GetHeaderBySlot(sszBlock.GetBeaconSlot())
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("fetch block: %w", err)
	}

	beaconHeader, err := header.ToScale()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("beacon header to scale: %w", err)
	}

	executionHeaderBranch, err := s.getExecutionHeaderBranch(sszBlock)
	if err != nil {
		return scale.HeaderUpdatePayload{}, err
	}

	var versionedExecutionPayloadHeader scale.VersionedExecutionPayloadHeader
	if s.protocol.DenebForked(slot) {
		executionPayloadScale, err := api.DenebExecutionPayloadToScale(sszBlock.ExecutionPayloadDeneb())
		if err != nil {
			return scale.HeaderUpdatePayload{}, err
		}
		versionedExecutionPayloadHeader = scale.VersionedExecutionPayloadHeader{Deneb: &executionPayloadScale}
	} else {
		executionPayloadScale, err := api.CapellaExecutionPayloadToScale(sszBlock.ExecutionPayloadCapella())
		if err != nil {
			return scale.HeaderUpdatePayload{}, err
		}
		versionedExecutionPayloadHeader = scale.VersionedExecutionPayloadHeader{Capella: &executionPayloadScale}
	}

	// If checkpoint not provided or slot == finalizedSlot there won't be an ancestry proof because the header state in question is also the finalized header
	if checkpoint == nil || sszBlock.GetBeaconSlot() == checkpoint.Slot {
		return scale.HeaderUpdatePayload{
			Header: beaconHeader,
			AncestryProof: scale.OptionAncestryProof{
				HasValue: false,
			},
			ExecutionHeader: versionedExecutionPayloadHeader,
			ExecutionBranch: executionHeaderBranch,
		}, nil
	}

	proofScale, err := s.getBlockHeaderAncestryProof(int(sszBlock.GetBeaconSlot()), blockRoot, checkpoint.BlockRootsTree)
	if err != nil {
		return scale.HeaderUpdatePayload{}, err
	}

	var displayProof []common.Hash
	for _, proof := range proofScale {
		displayProof = append(displayProof, common.HexToHash(proof.Hex()))
	}

	return scale.HeaderUpdatePayload{
		Header: beaconHeader,
		AncestryProof: scale.OptionAncestryProof{
			HasValue: true,
			Value: scale.AncestryProof{
				HeaderBranch:       proofScale,
				FinalizedBlockRoot: types.NewH256(checkpoint.FinalizedBlockRoot.Bytes()),
			},
		},
		ExecutionHeader: versionedExecutionPayloadHeader,
		ExecutionBranch: executionHeaderBranch,
	}, nil
}

func (s *Syncer) getBeaconStateAtSlot(slot uint64) (state.BeaconState, error) {
	var beaconState state.BeaconState
	log.WithField("slot", slot).Info("downloading state at slot")
	beaconData, err := s.Client.GetBeaconState(strconv.FormatUint(slot, 10))
	if err != nil {
		return beaconState, fmt.Errorf("fetch beacon state: %w", err)
	}

	return s.unmarshalBeaconState(slot, beaconData)
}

func (s *Syncer) unmarshalBeaconState(slot uint64, data []byte) (state.BeaconState, error) {
	var beaconState state.BeaconState
	isDeneb := s.protocol.DenebForked(slot)

	if isDeneb {
		beaconState = &state.BeaconStateDenebMainnet{}
	} else {
		beaconState = &state.BeaconStateCapellaMainnet{}
	}

	err := beaconState.UnmarshalSSZ(data)
	if err != nil {
		return beaconState, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	return beaconState, nil
}

// Sanity check the finalized and attested header are at 32 boundary blocks, so we can download the beacon state
func (s *Syncer) FindLatestAttestedHeadersAtInterval(initialSlot, lowestSlot uint64) (uint64, error) {
	slot := initialSlot

	for {
		finalizedSlot, attestedSlot, err := s.findValidUpdatePair(slot)
		if err != nil {
			if lowestSlot > slot {
				return 0, fmt.Errorf("unable to find valid slot")
			}

			slot -= s.protocol.Settings.SlotsInEpoch

			continue
		}

		log.WithFields(log.Fields{"attested": attestedSlot, "finalized": finalizedSlot}).Info("found boundary headers")
		return attestedSlot, nil
	}
}

// FindOldestAttestedHeaderAtInterval finds a set of headers (finalized and attested headers) that are at 32 boundary
// blocks (with a sync committee super majority signature), so we can download the beacon state.
func (s *Syncer) FindOldestAttestedHeaderAtInterval(initialSlot, highestSlot uint64) (uint64, error) {
	// special case where the finalized beacon state is not set at genesis
	if initialSlot == 0 {
		initialSlot = 2 * s.protocol.Settings.SlotsInEpoch
	}
	slot := initialSlot

	head, err := s.Client.GetHeaderAtHead()
	if err != nil {
		return 0, fmt.Errorf("get chain head: %w", err)
	}

	for {
		finalizedSlot, attestedSlot, err := s.findValidUpdatePair(slot)
		if err != nil {
			if highestSlot < slot || head.Slot < slot {
				return 0, fmt.Errorf("unable to find valid slot")
			}

			slot += s.protocol.Settings.SlotsInEpoch

			continue
		}

		log.WithFields(log.Fields{"attested": attestedSlot, "finalized": finalizedSlot}).Info("found boundary headers")
		return attestedSlot, nil
	}
}

func (s *Syncer) findValidUpdatePair(slot uint64) (uint64, uint64, error) {
	finalizedHeader, err := s.Client.GetHeaderBySlot(slot)
	if err != nil {
		return 0, 0, fmt.Errorf("get finalized slot: %d err: %w", slot, err)
	}

	attestedSlot := finalizedHeader.Slot + (s.protocol.Settings.SlotsInEpoch * 2)
	attestedHeader, err := s.Client.GetHeaderBySlot(attestedSlot)
	if err != nil {
		return 0, 0, fmt.Errorf("get attested slot: %d err: %w", attestedSlot, err)
	}

	nextHeader, err := s.FindBeaconHeaderWithBlockIncluded(attestedSlot + 1)
	if err != nil {
		return 0, 0, fmt.Errorf("get next header: %d err: %w", attestedSlot+1, err)
	}
	nextBlock, err := s.Client.GetBeaconBlockBySlot(nextHeader.Slot)
	if err != nil {
		return 0, 0, fmt.Errorf("get next block: %d err: %w", nextHeader.Slot, err)
	}

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(nextBlock.Data.Message.Body.SyncAggregate.SyncCommitteeBits)
	if err != nil {
		return 0, 0, fmt.Errorf("compute sync committee supermajority: %d err: %w", nextHeader.Slot, err)
	}
	if !superMajority {
		return 0, 0, fmt.Errorf("sync committee at slot not supermajority: %d", nextHeader.Slot)
	}

	return finalizedHeader.Slot, attestedHeader.Slot, nil
}

func (s *Syncer) GetLatestPossibleFinalizedUpdate(attestedSlot uint64, boundary uint64) (scale.Update, error) {
	attestedSlot, err := s.FindLatestAttestedHeadersAtInterval(attestedSlot, boundary)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	return s.GetFinalizedUpdateAtAttestedSlot(attestedSlot, boundary, false)
}

func (s *Syncer) GetFinalizedUpdateWithSyncCommittee(syncCommitteePeriod, lastFinalizedSlot uint64) (scale.Update, error) {
	slot := (syncCommitteePeriod) * s.protocol.Settings.SlotsInEpoch * s.protocol.Settings.EpochsPerSyncCommitteePeriod

	boundary := (syncCommitteePeriod + 1) * s.protocol.Settings.SlotsInEpoch * s.protocol.Settings.EpochsPerSyncCommitteePeriod

	attestedSlot, err := s.FindOldestAttestedHeaderAtInterval(slot, boundary)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	return s.GetFinalizedUpdateAtAttestedSlot(attestedSlot, boundary, true)
}

func (s *Syncer) GetFinalizedUpdateAtAttestedSlot(attestedSlot uint64, boundary uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
	var update scale.Update

	// Try getting beacon data from the API first
	data, err := s.getBeaconDataFromClient(attestedSlot)
	if err != nil {
		// If it fails, using the beacon store and look for a relevant finalized update
		data, err = s.getBeaconDataFromStore(attestedSlot, boundary, fetchNextSyncCommittee)
		if err != nil {
			return update, fmt.Errorf("fetch beacon data from api and data store failure: %w", err)
		}

		// The datastore may not have found the attested slot we wanted, but provided another valid one
		attestedSlot = data.AttestedSlot
	}

	// Finalized header proof
	stateTree, err := data.AttestedState.GetTree()
	if err != nil {
		return update, fmt.Errorf("get state tree: %w", err)
	}
	_ = stateTree.Hash() // necessary to populate the proof tree values
	finalizedHeaderProof, err := stateTree.Prove(FinalizedCheckpointGeneralizedIndex)
	if err != nil {
		return update, fmt.Errorf("get finalized header proof: %w", err)
	}

	var nextSyncCommitteeScale scale.OptionNextSyncCommitteeUpdatePayload
	if fetchNextSyncCommittee {
		nextSyncCommitteeProof, err := stateTree.Prove(NextSyncCommitteeGeneralizedIndex)
		if err != nil {
			return update, fmt.Errorf("get finalized header proof: %w", err)
		}

		nextSyncCommittee := data.AttestedState.GetSyncSyncCommittee()

		syncCommitteePubKeys, err := util.ByteArrayToPublicKeyArray(nextSyncCommittee.PubKeys)
		nextSyncCommitteeScale = scale.OptionNextSyncCommitteeUpdatePayload{
			HasValue: true,
			Value: scale.NextSyncCommitteeUpdatePayload{
				NextSyncCommittee: scale.SyncCommittee{
					Pubkeys:         syncCommitteePubKeys,
					AggregatePubkey: nextSyncCommittee.AggregatePubKey,
				},
				NextSyncCommitteeBranch: util.BytesBranchToScale(nextSyncCommitteeProof.Hashes),
			},
		}
	} else {
		nextSyncCommitteeScale = scale.OptionNextSyncCommitteeUpdatePayload{
			HasValue: false,
		}
	}

	blockRootsProof, err := s.GetBlockRootsFromState(data.FinalizedState)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch block roots: %w", err)
	}

	// Get the header at the slot
	header, err := s.Client.GetHeaderBySlot(attestedSlot)
	if err != nil {
		return update, fmt.Errorf("fetch header at slot: %w", err)
	}

	// Get the next block for the sync aggregate
	nextHeader, err := s.FindBeaconHeaderWithBlockIncluded(attestedSlot + 1)
	if err != nil {
		return update, fmt.Errorf("fetch block: %w", err)
	}

	nextBlock, err := s.Client.GetBeaconBlockBySlot(nextHeader.Slot)
	if err != nil {
		return update, fmt.Errorf("fetch block: %w", err)
	}

	nextBlockSlot, err := util.ToUint64(nextBlock.Data.Message.Slot)
	if err != nil {
		return update, fmt.Errorf("parse next block slot: %w", err)
	}

	scaleHeader, err := header.ToScale()
	if err != nil {
		return update, fmt.Errorf("convert header to scale: %w", err)
	}

	scaleFinalizedHeader, err := data.FinalizedHeader.ToScale()
	if err != nil {
		return update, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	syncAggregate := nextBlock.Data.Message.Body.SyncAggregate

	scaleSyncAggregate, err := syncAggregate.ToScale()
	if err != nil {
		return update, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	payload := scale.UpdatePayload{
		AttestedHeader:          scaleHeader,
		SyncAggregate:           scaleSyncAggregate,
		SignatureSlot:           types.U64(nextBlockSlot),
		NextSyncCommitteeUpdate: nextSyncCommitteeScale,
		FinalizedHeader:         scaleFinalizedHeader,
		FinalityBranch:          util.BytesBranchToScale(finalizedHeaderProof.Hashes),
		BlockRootsRoot:          blockRootsProof.Leaf,
		BlockRootsBranch:        blockRootsProof.Proof,
	}

	return scale.Update{
		Payload:                  payload,
		FinalizedHeaderBlockRoot: common.BytesToHash(data.FinalizedCheckPoint.Root),
		BlockRootsTree:           blockRootsProof.Tree,
	}, nil
}

func (s *Syncer) getBlockHeaderAncestryProof(slot int, blockRoot common.Hash, blockRootTree *ssz.Node) ([]types.H256, error) {
	maxSlotsPerHistoricalRoot := int(s.protocol.Settings.SlotsInEpoch * s.protocol.Settings.EpochsPerSyncCommitteePeriod)
	indexInArray := slot % maxSlotsPerHistoricalRoot
	leafIndex := maxSlotsPerHistoricalRoot + indexInArray

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

// Get the attested and finalized beacon states from the Beacon API.
func (s *Syncer) getBeaconDataFromClient(attestedSlot uint64) (finalizedUpdateContainer, error) {
	var response finalizedUpdateContainer
	var err error

	response.AttestedSlot = attestedSlot
	// Get the beacon data first since it is mostly likely to fail
	response.AttestedState, err = s.getBeaconStateAtSlot(attestedSlot)
	if err != nil {
		return response, fmt.Errorf("fetch attested header beacon state at slot %d: %w", attestedSlot, err)
	}

	response.FinalizedCheckPoint = *response.AttestedState.GetFinalizedCheckpoint()

	// Get the finalized header at the given slot state
	response.FinalizedHeader, err = s.Client.GetHeaderByBlockRoot(common.BytesToHash(response.FinalizedCheckPoint.Root))
	if err != nil {
		return response, fmt.Errorf("fetch header: %w", err)
	}

	response.FinalizedState, err = s.getBeaconStateAtSlot(response.FinalizedHeader.Slot)
	if err != nil {
		return response, fmt.Errorf("fetch attested header beacon state at slot %d: %w", attestedSlot, err)
	}

	return response, nil
}

// Get the best, latest finalized and attested beacon states including the slot provided in the finalized state block
// roots, from the Beacon store.
func (s *Syncer) getBeaconDataFromStore(slot, boundary uint64, findMin bool) (finalizedUpdateContainer, error) {
	response, err := s.getExactMatchFromStore(slot)
	if err != nil {
		response, err = s.getBestMatchBeaconDataFromStore(slot, boundary, findMin)
		if err != nil {
			return finalizedUpdateContainer{}, fmt.Errorf("unable to find exact slot or best other slot beacon data")
		}
	}

	return response, nil
}

func (s *Syncer) getBestMatchBeaconDataFromStore(slot, boundary uint64, findMin bool) (finalizedUpdateContainer, error) {
	var response finalizedUpdateContainer
	var err error

	data, err := s.store.FindBeaconStateWithinSyncPeriod(slot, boundary, findMin)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.AttestedSlot = data.AttestedSlot
	response.AttestedState, err = s.unmarshalBeaconState(data.AttestedSlot, data.AttestedBeaconState)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}
	response.FinalizedState, err = s.unmarshalBeaconState(data.FinalizedSlot, data.FinalizedBeaconState)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.FinalizedCheckPoint = *response.AttestedState.GetFinalizedCheckpoint()

	response.FinalizedHeader, err = s.Client.GetHeaderByBlockRoot(common.BytesToHash(response.FinalizedCheckPoint.Root))
	if err != nil {
		return response, fmt.Errorf("fetch header: %w", err)
	}

	return response, nil
}

func (s *Syncer) getExactMatchFromStore(slot uint64) (finalizedUpdateContainer, error) {
	var response finalizedUpdateContainer
	attestedStateData, err := s.store.GetBeaconStateData(slot)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.AttestedSlot = slot
	response.AttestedState, err = s.unmarshalBeaconState(slot, attestedStateData)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.FinalizedCheckPoint = *response.AttestedState.GetFinalizedCheckpoint()

	response.FinalizedHeader, err = s.Client.GetHeaderByBlockRoot(common.BytesToHash(response.FinalizedCheckPoint.Root))
	if err != nil {
		return response, fmt.Errorf("fetch header: %w", err)
	}

	finalizedStateData, err := s.store.GetBeaconStateData(response.FinalizedHeader.Slot)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.FinalizedState, err = s.unmarshalBeaconState(response.FinalizedHeader.Slot, finalizedStateData)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	return response, nil
}
