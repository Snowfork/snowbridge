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
	"github.com/snowfork/snowbridge/relayer/relays/util"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
)

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrSyncCommitteeNotSuperMajority              = errors.New("update received was not signed by supermajority")
	ErrNewerFinalizedHeaderAvailable              = errors.New("newer finalized header available, abandoning current request")
)

// StateServiceClient is an interface for the beacon state service HTTP client
type StateServiceClient interface {
	GetBlockRootProof(slot uint64) (*scale.BlockRootProof, error)
	GetFinalizedHeaderProof(slot uint64) ([]types.H256, error)
	GetSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error)
	Health() error
}

type Syncer struct {
	Client       api.BeaconAPI
	protocol     *protocol.Protocol
	stateService StateServiceClient
}

// New creates a Syncer with an optional beacon state service client.
// When stateService is provided, it handles all beacon state fetching with internal fallback logic
// (state service cache -> beacon API -> persistent store).
// When stateService is nil, the syncer falls back directly to the beacon API.
func New(client api.BeaconAPI, protocol *protocol.Protocol, stateService StateServiceClient) *Syncer {
	return &Syncer{
		Client:       client,
		protocol:     protocol,
		stateService: stateService,
	}
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

	// Get block roots proof
	blockRootsProof, err := s.GetBlockRoots(slot)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch block roots: %w", err)
	}

	// Get sync committee proof with pubkeys
	syncCommitteeProof, err := s.getSyncCommitteeProof(slot, "current")
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch sync committee proof: %w", err)
	}

	return scale.BeaconCheckpoint{
		Header: checkpoint.Payload.FinalizedHeader,
		CurrentSyncCommittee: scale.SyncCommittee{
			Pubkeys:         syncCommitteeProof.Pubkeys,
			AggregatePubkey: syncCommitteeProof.AggregatePubkey,
		},
		CurrentSyncCommitteeBranch: syncCommitteeProof.Proof,
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

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(committeeUpdate.SyncAggregate.SyncCommitteeBits)
	if err != nil {
		return scale.Update{}, fmt.Errorf("compute sync committee supermajority: %w", err)
	}
	if !superMajority {
		return scale.Update{}, ErrSyncCommitteeNotSuperMajority
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
	if s.stateService == nil {
		return scale.BlockRootProof{}, fmt.Errorf("state service is required but not configured")
	}

	// Retry with backoff if proof not ready
	// Max wait: 5+10+15+20+25+30*5 = 225 seconds (~4 minutes)
	// This is less than the ~6.4 minute finality period
	maxRetries := 10
	startTime := time.Now()
	maxWaitTime := 4 * time.Minute

	for i := 0; i < maxRetries; i++ {
		proof, err := s.stateService.GetBlockRootProof(slot)
		if err == nil {
			log.WithField("slot", slot).Debug("got block root proof from state service")
			return *proof, nil
		}

		// Check if it's a "not ready" error that we should retry
		if err.Error() == "proof not ready, please retry" {
			// Check if a newer finalized slot is available before retrying
			if newerSlot, hasNewer := s.checkForNewerFinalizedSlot(slot); hasNewer {
				log.WithFields(log.Fields{
					"requestedSlot": slot,
					"newerSlot":     newerSlot,
				}).Info("newer finalized header available, abandoning current request")
				return scale.BlockRootProof{}, ErrNewerFinalizedHeaderAvailable
			}

			// Check if we've been waiting too long - give up and let caller retry with fresh state
			if time.Since(startTime) > maxWaitTime {
				log.WithFields(log.Fields{
					"slot":     slot,
					"waitTime": time.Since(startTime),
				}).Info("proof wait timeout exceeded, abandoning request")
				return scale.BlockRootProof{}, ErrNewerFinalizedHeaderAvailable
			}

			waitTime := time.Duration(5*(i+1)) * time.Second
			if waitTime > 30*time.Second {
				waitTime = 30 * time.Second
			}
			log.WithFields(log.Fields{
				"slot":    slot,
				"attempt": i + 1,
				"wait":    waitTime,
			}).Info("proof not ready, retrying...")
			time.Sleep(waitTime)
			continue
		}

		// Other error - don't retry
		log.WithError(err).WithField("slot", slot).Error("state service failed to get block root proof")
		return scale.BlockRootProof{}, fmt.Errorf("state service failed: %w", err)
	}

	return scale.BlockRootProof{}, fmt.Errorf("state service retries exhausted for slot %d", slot)
}

// getSyncCommitteeProof fetches sync committee proof with pubkeys from state service with retry logic
func (s *Syncer) getSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error) {
	if s.stateService == nil {
		return nil, fmt.Errorf("state service is required but not configured")
	}

	maxRetries := 10
	startTime := time.Now()
	maxWaitTime := 4 * time.Minute

	for i := 0; i < maxRetries; i++ {
		proof, err := s.stateService.GetSyncCommitteeProof(slot, period)
		if err == nil {
			log.WithFields(log.Fields{"slot": slot, "period": period}).Debug("got sync committee proof from state service")
			return proof, nil
		}

		if err.Error() == "proof not ready, please retry" {
			// Check if a newer finalized slot is available before retrying
			if newerSlot, hasNewer := s.checkForNewerFinalizedSlot(slot); hasNewer {
				log.WithFields(log.Fields{
					"requestedSlot": slot,
					"newerSlot":     newerSlot,
				}).Info("newer finalized header available, abandoning sync committee proof request")
				return nil, ErrNewerFinalizedHeaderAvailable
			}

			// Check if we've been waiting too long
			if time.Since(startTime) > maxWaitTime {
				log.WithFields(log.Fields{
					"slot":     slot,
					"period":   period,
					"waitTime": time.Since(startTime),
				}).Info("sync committee proof wait timeout exceeded, abandoning request")
				return nil, ErrNewerFinalizedHeaderAvailable
			}

			waitTime := time.Duration(5*(i+1)) * time.Second
			if waitTime > 30*time.Second {
				waitTime = 30 * time.Second
			}
			log.WithFields(log.Fields{
				"slot":    slot,
				"period":  period,
				"attempt": i + 1,
				"wait":    waitTime,
			}).Info("sync committee proof not ready, retrying...")
			time.Sleep(waitTime)
			continue
		}

		log.WithError(err).WithFields(log.Fields{"slot": slot, "period": period}).Error("state service failed to get sync committee proof")
		return nil, fmt.Errorf("state service failed: %w", err)
	}
	return nil, fmt.Errorf("state service retries exhausted for sync committee proof at slot %d", slot)
}

// getFinalizedHeaderProof fetches finalized header proof from state service with retry logic
func (s *Syncer) getFinalizedHeaderProof(slot uint64) ([]types.H256, error) {
	if s.stateService == nil {
		return nil, fmt.Errorf("state service is required but not configured")
	}

	maxRetries := 10
	startTime := time.Now()
	maxWaitTime := 4 * time.Minute

	for i := 0; i < maxRetries; i++ {
		proof, err := s.stateService.GetFinalizedHeaderProof(slot)
		if err == nil {
			log.WithField("slot", slot).Debug("got finalized header proof from state service")
			return proof, nil
		}

		if err.Error() == "proof not ready, please retry" {
			// Check if a newer finalized slot is available before retrying
			if newerSlot, hasNewer := s.checkForNewerFinalizedSlot(slot); hasNewer {
				log.WithFields(log.Fields{
					"requestedSlot": slot,
					"newerSlot":     newerSlot,
				}).Info("newer finalized header available, abandoning finalized header proof request")
				return nil, ErrNewerFinalizedHeaderAvailable
			}

			// Check if we've been waiting too long
			if time.Since(startTime) > maxWaitTime {
				log.WithFields(log.Fields{
					"slot":     slot,
					"waitTime": time.Since(startTime),
				}).Info("finalized header proof wait timeout exceeded, abandoning request")
				return nil, ErrNewerFinalizedHeaderAvailable
			}

			waitTime := time.Duration(5*(i+1)) * time.Second
			if waitTime > 30*time.Second {
				waitTime = 30 * time.Second
			}
			log.WithFields(log.Fields{
				"slot":    slot,
				"attempt": i + 1,
				"wait":    waitTime,
			}).Info("finalized header proof not ready, retrying...")
			time.Sleep(waitTime)
			continue
		}

		log.WithError(err).WithField("slot", slot).Error("state service failed to get finalized header proof")
		return nil, fmt.Errorf("state service failed: %w", err)
	}
	return nil, fmt.Errorf("state service retries exhausted for finalized header proof at slot %d", slot)
}

// checkForNewerFinalizedSlot checks if a newer finalized slot is available than the one being requested.
// Returns the newer slot and true if a newer one exists, otherwise returns 0 and false.
func (s *Syncer) checkForNewerFinalizedSlot(requestedSlot uint64) (uint64, bool) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		// If we can't check, assume no newer slot is available
		log.WithError(err).Debug("failed to check for newer finalized slot")
		return 0, false
	}

	currentFinalizedSlot, err := strconv.ParseUint(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		log.WithError(err).Debug("failed to parse current finalized slot")
		return 0, false
	}

	if currentFinalizedSlot > requestedSlot {
		return currentFinalizedSlot, true
	}

	return 0, false
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

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(finalizedUpdate.Data.SyncAggregate.SyncCommitteeBits)
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
		log.WithFields(log.Fields{
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
	if forkVersion == protocol.Fulu || forkVersion == protocol.Electra {
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

func (s *Syncer) UnmarshalBeaconState(slot uint64, data []byte) (state.BeaconState, error) {
	var beaconState state.BeaconState
	forkVersion := s.protocol.ForkVersion(slot)
	if forkVersion == protocol.Fulu {
		beaconState = &state.BeaconStateFulu{}
	} else if forkVersion == protocol.Electra {
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
	if s.stateService == nil {
		return scale.Update{}, fmt.Errorf("state service is required but not configured")
	}

	return s.getFinalizedUpdateFromStateService(minSlot, maxSlot, fetchNextSyncCommittee)
}

// getFinalizedUpdateFromStateService gets finalized update using proofs from state service
// This path never handles raw beacon state data
func (s *Syncer) getFinalizedUpdateFromStateService(minSlot, maxSlot uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
	attestedSlot, err := s.FindValidAttestedHeader(minSlot, maxSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	// Get finalized header from light client API
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.Update{}, fmt.Errorf("get finalized update from API: %w", err)
	}

	finalizedSlot, err := util.ToUint64(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("parse finalized slot: %w", err)
	}

	log.WithFields(log.Fields{"finalizedSlot": finalizedSlot, "attestedSlot": attestedSlot}).Info("found slot pair for finalized update")

	// Get proofs from state service
	finalityProof, err := s.getFinalizedHeaderProof(attestedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("get finalized header proof: %w", err)
	}

	var nextSyncCommitteeScale scale.OptionNextSyncCommitteeUpdatePayload
	if fetchNextSyncCommittee {
		syncCommitteeProof, err := s.getSyncCommitteeProof(attestedSlot, "next")
		if err != nil {
			return scale.Update{}, fmt.Errorf("get next sync committee proof: %w", err)
		}
		nextSyncCommitteeScale = scale.OptionNextSyncCommitteeUpdatePayload{
			HasValue: true,
			Value: scale.NextSyncCommitteeUpdatePayload{
				NextSyncCommittee: scale.SyncCommittee{
					Pubkeys:         syncCommitteeProof.Pubkeys,
					AggregatePubkey: syncCommitteeProof.AggregatePubkey,
				},
				NextSyncCommitteeBranch: syncCommitteeProof.Proof,
			},
		}
	} else {
		nextSyncCommitteeScale = scale.OptionNextSyncCommitteeUpdatePayload{
			HasValue: false,
		}
	}

	blockRootsProof, err := s.GetBlockRoots(finalizedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch block roots: %w", err)
	}

	// Get headers from beacon API
	header, err := s.Client.GetHeaderBySlot(attestedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch header at slot: %w", err)
	}

	finalizedHeader, err := s.Client.GetHeaderBySlot(finalizedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch finalized header at slot: %w", err)
	}

	// Get the next block for the sync aggregate
	nextHeader, err := s.FindBeaconHeaderWithBlockIncluded(attestedSlot + 1)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch block: %w", err)
	}

	nextBlock, err := s.Client.GetBeaconBlockBySlot(nextHeader.Slot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch block: %w", err)
	}

	nextBlockSlot, err := util.ToUint64(nextBlock.Data.Message.Slot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("parse next block slot: %w", err)
	}

	scaleHeader, err := header.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert header to scale: %w", err)
	}

	scaleFinalizedHeader, err := finalizedHeader.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert finalized header to scale: %w", err)
	}

	syncAggregate := nextBlock.Data.Message.Body.SyncAggregate
	scaleSyncAggregate, err := syncAggregate.ToScale()
	if err != nil {
		return scale.Update{}, fmt.Errorf("convert sync aggregate to scale: %w", err)
	}

	superMajority, err := s.protocol.SyncCommitteeSuperMajority(syncAggregate.SyncCommitteeBits)
	if err != nil {
		return scale.Update{}, fmt.Errorf("compute sync committee supermajority: %w", err)
	}
	if !superMajority {
		return scale.Update{}, ErrSyncCommitteeNotSuperMajority
	}

	// Get finalized block root from beacon API
	finalizedBlockRoot, err := s.Client.GetBeaconBlockRoot(finalizedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("get finalized block root: %w", err)
	}

	payload := scale.UpdatePayload{
		AttestedHeader:          scaleHeader,
		SyncAggregate:           scaleSyncAggregate,
		SignatureSlot:           types.U64(nextBlockSlot),
		NextSyncCommitteeUpdate: nextSyncCommitteeScale,
		FinalizedHeader:         scaleFinalizedHeader,
		FinalityBranch:          finalityProof,
		BlockRootsRoot:          blockRootsProof.Leaf,
		BlockRootsBranch:        blockRootsProof.Proof,
	}

	return scale.Update{
		Payload:                  payload,
		FinalizedHeaderBlockRoot: finalizedBlockRoot,
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
