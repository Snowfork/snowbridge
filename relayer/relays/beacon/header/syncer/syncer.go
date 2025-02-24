package syncer

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strconv"
	"time"

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

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrBeaconStateUnavailable                     = errors.New("beacon state object not available yet")
	ErrSyncCommitteeNotSuperMajority              = errors.New("update received was not signed by supermajority")
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
	retries := 5
	bootstrap, err := s.getCheckpoint()
	if err != nil {
		for retries > 0 {
			retries = retries - 1
			bootstrap, err = s.getCheckpoint()
			if err != nil {
				log.WithError(err).Info("retry bootstrap, sleeping")
				time.Sleep(10 * time.Second)
				continue
			}
			break
		}
	}
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

	return scale.BeaconCheckpoint{
		Header:                     header,
		CurrentSyncCommittee:       syncCommittee,
		CurrentSyncCommitteeBranch: util.ProofBranchToScale(bootstrap.Data.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             types.H256(genesis.ValidatorsRoot),
		BlockRootsRoot:             blockRootsProof.Leaf,
		BlockRootsBranch:           blockRootsProof.Proof,
	}, nil
}

func (s *Syncer) getCheckpoint() (api.BootstrapResponse, error) {
	checkpoint, err := s.Client.GetFinalizedCheckpoint()
	if err != nil {
		return api.BootstrapResponse{}, fmt.Errorf("get finalized checkpoint: %w", err)
	}

	bootstrap, err := s.Client.GetBootstrap(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return api.BootstrapResponse{}, fmt.Errorf("get bootstrap: %w", err)
	}

	return bootstrap, err
}

func (s *Syncer) GetCheckpointFromFile(file string) (scale.BeaconCheckpoint, error) {
	type CheckPointResponse struct {
		Header                     api.BeaconHeader          `json:"header"`
		CurrentSyncCommittee       api.SyncCommitteeResponse `json:"current_sync_committee"`
		CurrentSyncCommitteeBranch []string                  `json:"current_sync_committee_branch"`
		ValidatorsRoot             string                    `json:"validators_root"`
		BlockRootsRoot             string                    `json:"block_roots_root"`
		BlockRootsRootBranch       []string                  `json:"block_roots_branch"`
	}
	var response CheckPointResponse

	byteValue, err := os.ReadFile(file)
	if err != nil {
		return scale.BeaconCheckpoint{}, err
	}

	err = json.Unmarshal(byteValue, &response)
	if err != nil {
		return scale.BeaconCheckpoint{}, err
	}

	header, err := response.Header.ToScale()
	if err != nil {
		return scale.BeaconCheckpoint{}, err
	}

	currentSyncCommittee, err := response.CurrentSyncCommittee.ToScale()
	if err != nil {
		return scale.BeaconCheckpoint{}, err
	}

	return scale.BeaconCheckpoint{
		Header:                     header,
		CurrentSyncCommittee:       currentSyncCommittee,
		CurrentSyncCommitteeBranch: util.ProofBranchToScale(response.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             types.H256(common.HexToHash(response.ValidatorsRoot)),
		BlockRootsRoot:             types.H256(common.HexToHash(response.BlockRootsRoot)),
		BlockRootsBranch:           util.ProofBranchToScale(response.BlockRootsRootBranch),
	}, nil
}

func (s *Syncer) GetCheckpointAtSlot(slot uint64) (scale.BeaconCheckpoint, error) {
	checkpoint, err := s.GetFinalizedUpdateAtAttestedSlot(slot, slot, false)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get finalized update at slot: %w", err)
	}
	// In case the update returns a different finalized update that requested (missed blocks, etc)
	slot = uint64(checkpoint.Payload.FinalizedHeader.Slot)

	genesis, err := s.Client.GetGenesis()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get genesis: %w", err)
	}

	finalizedState, err := s.getBeaconStateAtSlot(slot)

	blockRootsProof, err := s.GetBlockRootsFromState(finalizedState)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncCommittee := finalizedState.GetCurrentSyncCommittee()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("convert sync committee to scale: %w", err)
	}

	stateTree, err := finalizedState.GetTree()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get state tree: %w", err)
	}

	_ = stateTree.Hash() // necessary to populate the proof tree values

	proof, err := stateTree.Prove(s.protocol.CurrentSyncCommitteeGeneralizedIndex(uint64(checkpoint.Payload.FinalizedHeader.Slot)))
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get block roof proof: %w", err)
	}

	pubkeys, err := util.ByteArrayToPublicKeyArray(syncCommittee.PubKeys)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("bytes to pubkey array: %w", err)
	}

	return scale.BeaconCheckpoint{
		Header: checkpoint.Payload.FinalizedHeader,
		CurrentSyncCommittee: scale.SyncCommittee{
			Pubkeys:         pubkeys,
			AggregatePubkey: syncCommittee.AggregatePubKey,
		},
		CurrentSyncCommitteeBranch: util.BytesBranchToScale(proof.Hashes),
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
		update, err = s.GetFinalizedUpdateWithSyncCommittee(period)
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
		return scale.Update{}, fmt.Errorf("fetch block roots proof: %w", err)
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

	data, err := s.getBeaconState(slot)
	if err != nil {
		return blockRootProof, fmt.Errorf("fetch beacon state: %w", err)
	}

	forkVersion := s.protocol.ForkVersion(slot)

	blockRootsContainer = &state.BlockRootsContainerMainnet{}
	if forkVersion == protocol.Electra {
		beaconState = &state.BeaconStateElectra{}
	} else {
		beaconState = &state.BeaconStateDenebMainnet{}
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

	proof, err := stateTree.Prove(s.protocol.BlockRootGeneralizedIndex(slot))
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

	proof, err := stateTree.Prove(s.protocol.BlockRootGeneralizedIndex(beaconState.GetSlot()))
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

func (s *Syncer) GetFinalizedHeader() (scale.BeaconHeader, error) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("fetch finalized update: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	return finalizedHeader, nil
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

	signatureBlock, err := s.Client.GetBeaconBlockBySlot(signatureSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("get signature block: %w", err)
	}

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(signatureBlock.Data.Message.Body.SyncAggregate.SyncCommitteeBits)
	if err != nil {
		return scale.Update{}, fmt.Errorf("compute sync committee supermajority: %d err: %w", signatureSlot, err)
	}
	if !superMajority {
		return scale.Update{}, ErrSyncCommitteeNotSuperMajority
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
	blockBytes, err := s.Client.GetBeaconBlockBytes(blockRoot)
	if err != nil {
		return update, fmt.Errorf("fetch block: %w", err)
	}

	header, err := s.Client.GetHeaderByBlockRoot(blockRoot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("fetch block: %w", err)
	}

	slot := header.Slot

	var signedBlock state.SignedBeaconBlock
	forkVersion := s.protocol.ForkVersion(slot)
	if forkVersion == protocol.Electra {
		signedBlock = &state.SignedBeaconBlockElectra{}
	} else {
		signedBlock = &state.SignedBeaconBlockDeneb{}
	}

	err = signedBlock.UnmarshalSSZ(blockBytes)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("unmarshal block ssz: %w", err)
	}

	beaconHeader, err := header.ToScale()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("beacon header to scale: %w", err)
	}

	beaconBlock := signedBlock.GetBlock()
	executionHeaderBranch, err := s.getExecutionHeaderBranch(beaconBlock)
	if err != nil {
		return scale.HeaderUpdatePayload{}, err
	}

	var versionedExecutionPayloadHeader scale.VersionedExecutionPayloadHeader
	executionPayloadScale, err := api.DenebExecutionPayloadToScale(beaconBlock.ExecutionPayloadDeneb())
	if err != nil {
		return scale.HeaderUpdatePayload{}, err
	}
	versionedExecutionPayloadHeader = scale.VersionedExecutionPayloadHeader{Deneb: &executionPayloadScale}

	// If checkpoint not provided or slot == finalizedSlot there won't be an ancestry proof because the header state in question is also the finalized header
	if checkpoint == nil || beaconBlock.GetBeaconSlot() == checkpoint.Slot {
		return scale.HeaderUpdatePayload{
			Header: beaconHeader,
			AncestryProof: scale.OptionAncestryProof{
				HasValue: false,
			},
			ExecutionHeader: versionedExecutionPayloadHeader,
			ExecutionBranch: executionHeaderBranch,
		}, nil
	}

	proofScale, err := s.getBlockHeaderAncestryProof(int(beaconBlock.GetBeaconSlot()), blockRoot, checkpoint.BlockRootsTree)
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
	beaconData, err := s.getBeaconState(slot)
	if err != nil {
		return beaconState, fmt.Errorf("fetch beacon state: %w", err)
	}

	return s.UnmarshalBeaconState(slot, beaconData)
}

func (s *Syncer) UnmarshalBeaconState(slot uint64, data []byte) (state.BeaconState, error) {
	var beaconState state.BeaconState
	forkVersion := s.protocol.ForkVersion(slot)
	if forkVersion == protocol.Electra {
		beaconState = &state.BeaconStateElectra{}
	} else {
		beaconState = &state.BeaconStateDenebMainnet{}
	}

	err := beaconState.UnmarshalSSZ(data)
	if err != nil {
		return beaconState, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	return beaconState, nil
}

// FindValidAttestedHeader Find a valid beacon header attested and finalized header pair.
func (s *Syncer) FindValidAttestedHeader(minSlot, maxSlot uint64) (uint64, error) {
	var slot uint64
	// make sure the starting slot is in a multiple of 32
	if minSlot%32 == 0 {
		slot = minSlot
	} else {
		slot = ((minSlot / s.protocol.Settings.SlotsInEpoch) + 1) * s.protocol.Settings.SlotsInEpoch
	}

	for {
		finalizedSlot, attestedSlot, err := s.findValidUpdatePair(slot)
		if err != nil {
			if slot > maxSlot {
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

func (s *Syncer) ValidatePair(finalizedSlot, attestedSlot uint64, attestedState state.BeaconState) error {
	finalizedCheckpoint := attestedState.GetFinalizedCheckpoint()
	finalizedHeader, err := s.Client.GetHeaderByBlockRoot(common.BytesToHash(finalizedCheckpoint.Root))
	if err != nil {
		return fmt.Errorf("unable to download finalized header from attested state")
	}

	if finalizedHeader.Slot != finalizedSlot {
		return fmt.Errorf("finalized state in attested state does not match provided finalized state, attested state finalized slot: %d, finalized slot provided: %d", finalizedHeader.Slot, finalizedSlot)
	}

	nextHeader, err := s.FindBeaconHeaderWithBlockIncluded(attestedSlot + 1)
	if err != nil {
		return fmt.Errorf("get sync aggregate header: %d err: %w", attestedSlot+1, err)
	}
	nextBlock, err := s.Client.GetBeaconBlockBySlot(nextHeader.Slot)
	if err != nil {
		return fmt.Errorf("get sync aggregate block: %d err: %w", nextHeader.Slot, err)
	}

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(nextBlock.Data.Message.Body.SyncAggregate.SyncCommitteeBits)
	if err != nil {
		return fmt.Errorf("compute sync committee supermajority: %d err: %w", nextHeader.Slot, err)
	}
	if !superMajority {
		return fmt.Errorf("sync committee at slot not supermajority: %d", nextHeader.Slot)
	}

	return nil
}

func (s *Syncer) GetFinalizedUpdateWithSyncCommittee(syncCommitteePeriod uint64) (scale.Update, error) {
	minSlot := syncCommitteePeriod * s.protocol.SlotsPerHistoricalRoot
	maxSlot := ((syncCommitteePeriod + 1) * s.protocol.SlotsPerHistoricalRoot) - s.protocol.Settings.SlotsInEpoch // just before the new sync committee boundary

	attestedSlot, err := s.FindValidAttestedHeader(minSlot, maxSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	return s.GetFinalizedUpdateAtAttestedSlot(attestedSlot, maxSlot, true)
}

func (s *Syncer) GetFinalizedUpdateAtAttestedSlot(minSlot, maxSlot uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
	var update scale.Update

	attestedSlot, err := s.FindValidAttestedHeader(minSlot, maxSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	// Try getting beacon data from the API first
	data, err := s.getBeaconDataFromClient(attestedSlot)
	if err != nil {
		log.WithError(err).Warn("unable to fetch beacon data from API, trying beacon store")
		// If it fails, using the beacon store and look for a relevant finalized update
		for {
			if minSlot > maxSlot {
				return update, fmt.Errorf("find beacon state store options exhausted: %w", err)
			}

			data, err = s.getBestMatchBeaconDataFromStore(minSlot, maxSlot)
			if err != nil {
				return update, fmt.Errorf("fetch beacon data from api and data store failure: %w", err)
			}

			err = s.ValidatePair(data.FinalizedHeader.Slot, data.AttestedSlot, data.AttestedState)
			if err != nil {
				minSlot = data.FinalizedHeader.Slot + 1
				log.WithError(err).WithField("minSlot", minSlot).Warn("pair retrieved from database invalid")
				continue
			}

			// The datastore may not have found the attested slot we wanted, but provided another valid one
			attestedSlot = data.AttestedSlot
			break
		}
	}

	log.WithFields(log.Fields{"finalizedSlot": data.FinalizedHeader.Slot, "attestedSlot": data.AttestedSlot}).Info("found slot pair for finalized update")
	// Finalized header proof
	stateTree, err := data.AttestedState.GetTree()
	if err != nil {
		return update, fmt.Errorf("get state tree: %w", err)
	}
	_ = stateTree.Hash() // necessary to populate the proof tree values
	finalizedHeaderProof, err := stateTree.Prove(s.protocol.FinalizedCheckpointGeneralizedIndex(attestedSlot))
	if err != nil {
		return update, fmt.Errorf("get finalized header proof: %w", err)
	}

	var nextSyncCommitteeScale scale.OptionNextSyncCommitteeUpdatePayload
	if fetchNextSyncCommittee {
		nextSyncCommitteeProof, err := stateTree.Prove(s.protocol.NextSyncCommitteeGeneralizedIndex(attestedSlot))
		if err != nil {
			return update, fmt.Errorf("get finalized header proof: %w", err)
		}

		nextSyncCommittee := data.AttestedState.GetNextSyncCommittee()

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
	maxSlotsPerHistoricalRoot := int(s.protocol.SlotsPerHistoricalRoot)
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

	proof, err := tree.Prove(s.protocol.ExecutionPayloadGeneralizedIndex(block.GetBeaconSlot()))

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

func (s *Syncer) getBestMatchBeaconDataFromStore(minSlot, maxSlot uint64) (finalizedUpdateContainer, error) {
	var response finalizedUpdateContainer
	var err error

	data, err := s.store.FindBeaconStateWithinRange(minSlot, maxSlot)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.AttestedSlot = data.AttestedSlot
	response.AttestedState, err = s.UnmarshalBeaconState(data.AttestedSlot, data.AttestedBeaconState)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}
	response.FinalizedState, err = s.UnmarshalBeaconState(data.FinalizedSlot, data.FinalizedBeaconState)
	if err != nil {
		return finalizedUpdateContainer{}, err
	}

	response.FinalizedCheckPoint = *response.AttestedState.GetFinalizedCheckpoint()

	response.FinalizedHeader, err = s.Client.GetHeaderByBlockRoot(common.BytesToHash(response.FinalizedCheckPoint.Root))
	if err != nil {
		return response, fmt.Errorf("fetch header: %w", err)
	}

	if response.FinalizedHeader.Slot != response.FinalizedState.GetSlot() {
		return response, fmt.Errorf("finalized slot in state does not match attested finalized state: %w", err)
	}

	return response, nil
}

func (s *Syncer) getBeaconState(slot uint64) ([]byte, error) {
	data, apiErr := s.Client.GetBeaconState(strconv.FormatUint(slot, 10))
	if apiErr != nil {
		var storeErr error
		data, storeErr = s.store.GetBeaconStateData(slot)
		if storeErr != nil {
			log.WithFields(log.Fields{"apiError": apiErr, "storeErr": storeErr}).Warn("fetch beacon state from api and store failed")
			return nil, ErrBeaconStateUnavailable
		}
	}
	return data, nil
}
