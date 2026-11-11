package syncer

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	beaconerrors "github.com/snowfork/snowbridge/relayer/relays/beacon/errors"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/execution"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

const stateServiceErrorThreshold = 3 * time.Minute

var (
	ErrCommitteeUpdateHeaderInDifferentSyncPeriod = errors.New("sync committee in different sync period")
	ErrSyncCommitteeNotSuperMajority              = errors.New("update received was not signed by supermajority")
	ErrNewerFinalizedHeaderAvailable              = errors.New("newer finalized header available, abandoning current request")
)

type StateServiceClient interface {
	GetBlockRootProof(slot uint64) (*scale.BlockRootProof, error)
	GetFinalizedHeaderProof(slot uint64) ([]types.H256, error)
	GetFinalizedHeaderCheckpointRoot(slot uint64) (common.Hash, error)
	GetSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error)
	Health() error
}

type Syncer struct {
	Client       api.BeaconAPI
	protocol     *protocol.Protocol
	stateService StateServiceClient
	executionClient execution.HeaderFetcher // Gloas execution proofs only
	blockRootsMu    sync.Mutex
	blockRootsCache map[uint64]scale.BlockRootProof
}

func New(client api.BeaconAPI, protocol *protocol.Protocol, stateService StateServiceClient) *Syncer {
	return &Syncer{
		Client:          client,
		protocol:        protocol,
		stateService:    stateService,
		blockRootsCache: make(map[uint64]scale.BlockRootProof),
	}
}

func (s *Syncer) WithExecutionClient(client execution.HeaderFetcher) *Syncer {
	s.executionClient = client
	return s
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
	// No light-client bootstrap on some Gloas testnets; build checkpoint from header + state service.
	header, err := s.FindBeaconHeaderWithBlockIncluded(slot)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("find header at slot %d: %w", slot, err)
	}
	slot = header.Slot

	headerScale := scale.BeaconHeader{
		Slot:          types.U64(header.Slot),
		ProposerIndex: types.U64(header.ProposerIndex),
		ParentRoot:    types.NewH256(header.ParentRoot),
		StateRoot:     types.NewH256(header.StateRoot),
		BodyRoot:      types.NewH256(header.BodyRoot),
	}

	genesis, err := s.Client.GetGenesis()
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("get genesis: %w", err)
	}

	blockRootsProof, err := s.GetBlockRoots(slot)
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch block roots: %w", err)
	}

	syncCommitteeProof, err := s.getSyncCommitteeProof(slot, "current")
	if err != nil {
		return scale.BeaconCheckpoint{}, fmt.Errorf("fetch sync committee proof: %w", err)
	}

	return scale.BeaconCheckpoint{
		Header: headerScale,
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

// allowFallback: retry hard errors then reconstruct block roots; else Fatal after threshold.
func (s *Syncer) retryWithBackoff(slot uint64, proofType string, allowFallback bool, fetchFn func() error) error {
	const maxRetries = 10
	startTime := time.Now()
	var lastErr error

	for i := 0; i < maxRetries; i++ {
		err := fetchFn()
		if err == nil {
			return nil
		}
		lastErr = err

		proofNotReady := errors.Is(err, beaconerrors.ErrProofNotReady)
		if proofNotReady || allowFallback {
			if newerSlot, hasNewer := s.checkForNewerFinalizedSlot(slot); hasNewer {
				log.WithFields(log.Fields{
					"requestedSlot": slot,
					"newerSlot":     newerSlot,
					"proofType":     proofType,
				}).Info("newer finalized header available, abandoning current request")
				return ErrNewerFinalizedHeaderAvailable
			}

			if time.Since(startTime) > stateServiceErrorThreshold {
				fields := log.Fields{
					"slot":      slot,
					"proofType": proofType,
					"waitTime":  time.Since(startTime),
				}
				if allowFallback {
					log.WithError(err).WithFields(fields).Warn("beacon state service still failing after 3 minutes; falling back")
					return err
				}
				log.WithFields(fields).Fatal("beacon state service proof not ready for over 3 minutes, restarting relayer")
			}

			waitTime := time.Duration(5*(i+1)) * time.Second
			if waitTime > 30*time.Second {
				waitTime = 30 * time.Second
			}
			fields := log.Fields{
				"slot":      slot,
				"proofType": proofType,
				"attempt":   i + 1,
				"wait":      waitTime,
			}
			if proofNotReady {
				log.WithFields(fields).Info("proof not ready, retrying...")
			} else {
				log.WithError(err).WithFields(fields).Warn("beacon state service error, retrying before fallback...")
			}
			time.Sleep(waitTime)
			continue
		}

		log.WithError(err).WithFields(log.Fields{
			"slot":      slot,
			"proofType": proofType,
		}).Fatal("beacon state service returned error, restarting relayer")
	}

	fields := log.Fields{
		"slot":      slot,
		"proofType": proofType,
	}
	if allowFallback {
		log.WithError(lastErr).WithFields(fields).Warn("beacon state service retries exhausted; falling back")
		if lastErr != nil {
			return lastErr
		}
		return fmt.Errorf("%w: retries exhausted", beaconerrors.ErrProofNotReady)
	}
	log.WithFields(fields).Fatal("beacon state service retries exhausted, restarting relayer")
	return nil
}

func (s *Syncer) GetBlockRoots(slot uint64) (scale.BlockRootProof, error) {
	// Historical slots: state service often 503; reconstruct block_roots from headers.
	if s.stateService != nil {
		if s.isNearCurrentFinalized(slot) {
			var result *scale.BlockRootProof
			err := s.retryWithBackoff(slot, "block root proof", true, func() error {
				proof, err := s.stateService.GetBlockRootProof(slot)
				if err != nil {
					return err
				}
				log.WithField("slot", slot).Debug("got block root proof from state service")
				result = proof
				return nil
			})
			if err == nil {
				return *result, nil
			}
			if errors.Is(err, ErrNewerFinalizedHeaderAvailable) {
				return scale.BlockRootProof{}, err
			}
			log.WithError(err).WithField("slot", slot).Warn("state service block root proof failed near tip; trying header reconstruction")
		} else {
			proof, err := s.stateService.GetBlockRootProof(slot)
			if err == nil {
				log.WithField("slot", slot).Debug("got block root proof from state service")
				return *proof, nil
			}
			if !errors.Is(err, beaconerrors.ErrProofNotReady) {
				log.WithError(err).WithField("slot", slot).Warn("state service block root proof error; reconstructing from headers")
			} else {
				log.WithField("slot", slot).Info("historical block root proof not in state service cache; reconstructing from headers")
			}
		}
	}

	return s.reconstructBlockRootsFromHeaders(slot)
}

func (s *Syncer) isNearCurrentFinalized(slot uint64) bool {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return true // fail open: tip backoff
	}
	currentFinalizedSlot, err := strconv.ParseUint(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return true
	}
	if slot > currentFinalizedSlot {
		return true
	}
	nearTipWindow := s.protocol.Settings.SlotsInEpoch * 2
	return currentFinalizedSlot-slot <= nearTipWindow
}

// Rebuild block_roots ring from headers; skipped slots forward-fill like process_slot.
func (s *Syncer) reconstructBlockRootsFromHeaders(stateSlot uint64) (scale.BlockRootProof, error) {
	s.blockRootsMu.Lock()
	if cached, ok := s.blockRootsCache[stateSlot]; ok {
		s.blockRootsMu.Unlock()
		return cached, nil
	}
	s.blockRootsMu.Unlock()

	proof, err := s.reconstructBlockRootsFromHeadersUncached(stateSlot)
	if err != nil {
		return scale.BlockRootProof{}, err
	}

	s.blockRootsMu.Lock()
	if s.blockRootsCache == nil {
		s.blockRootsCache = make(map[uint64]scale.BlockRootProof)
	}
	s.blockRootsCache[stateSlot] = proof
	if len(s.blockRootsCache) > 64 {
		for k := range s.blockRootsCache {
			if k != stateSlot {
				delete(s.blockRootsCache, k)
				break
			}
		}
	}
	s.blockRootsMu.Unlock()
	return proof, nil
}

func (s *Syncer) reconstructBlockRootsFromHeadersUncached(stateSlot uint64) (scale.BlockRootProof, error) {
	n := s.protocol.SlotsPerHistoricalRoot
	if stateSlot < n {
		return scale.BlockRootProof{}, fmt.Errorf("state slot %d smaller than slots per historical root %d", stateSlot, n)
	}

	startSlot := stateSlot - n
	log.WithFields(log.Fields{
		"stateSlot":  stateSlot,
		"startSlot":  startSlot,
		"numHeaders": n,
	}).Info("reconstructing block_roots tree from beacon headers")

	type slotRoot struct {
		slot uint64
		root common.Hash
		err  error
	}

	const workers = 6
	const targetRPS = 12
	interval := time.Second / time.Duration(targetRPS)
	pace := time.NewTicker(interval)
	defer pace.Stop()

	slots := make(chan uint64, workers*2)
	results := make(chan slotRoot, workers*2)

	fetchRoot := func(slot uint64) (common.Hash, error) {
		var lastErr error
		for attempt := 0; attempt < 12; attempt++ {
			<-pace.C
			root, err := s.Client.GetBeaconBlockRoot(slot)
			if err == nil {
				return root, nil
			}
			lastErr = err
			if errors.Is(err, api.ErrNotFound) {
				<-pace.C
				header, herr := s.Client.GetHeaderBySlot(slot)
				if herr != nil {
					if errors.Is(herr, api.ErrNotFound) {
						return common.Hash{}, api.ErrNotFound
					}
					lastErr = herr
				} else {
					scaleHeader, herr := header.ToScale()
					if herr != nil {
						return common.Hash{}, herr
					}
					htr, herr := scaleHeader.ToSSZ().HashTreeRoot()
					if herr != nil {
						return common.Hash{}, herr
					}
					return common.BytesToHash(htr[:]), nil
				}
			}
			errStr := strings.ToLower(lastErr.Error())
			if strings.Contains(errStr, "429") ||
				strings.Contains(errStr, "too many") ||
				strings.Contains(errStr, "http status not ok") ||
				strings.Contains(errStr, "do http request") {
				wait := time.Duration(1<<attempt) * time.Second
				if wait > 20*time.Second {
					wait = 20 * time.Second
				}
				time.Sleep(wait)
				continue
			}
			return common.Hash{}, lastErr
		}
		return common.Hash{}, lastErr
	}

	var fetchGroup errgroup.Group
	for i := 0; i < workers; i++ {
		fetchGroup.Go(func() error {
			for slot := range slots {
				root, err := fetchRoot(slot)
				results <- slotRoot{slot: slot, root: root, err: err}
			}
			return nil
		})
	}

	go func() {
		for slot := startSlot; slot < stateSlot; slot++ {
			slots <- slot
		}
		close(slots)
		_ = fetchGroup.Wait()
		close(results)
	}()

	raw := make([]common.Hash, n)
	var fatal error
	var mu sync.Mutex
	misses := 0
	for r := range results {
		if r.err != nil {
			if errors.Is(r.err, api.ErrNotFound) {
				mu.Lock()
				misses++
				mu.Unlock()
				continue
			}
			mu.Lock()
			if fatal == nil {
				fatal = fmt.Errorf("fetch block root for slot %d: %w", r.slot, r.err)
			}
			mu.Unlock()
			continue
		}
		idx := r.slot - startSlot
		raw[idx] = r.root
	}
	if fatal != nil {
		return scale.BlockRootProof{}, fatal
	}

	blockRoots := make([][]byte, n)
	var prev []byte
	filled := 0
	for i := uint64(0); i < n; i++ {
		slot := startSlot + i
		if raw[i] != (common.Hash{}) {
			b := make([]byte, 32)
			copy(b, raw[i].Bytes())
			prev = b
			blockRoots[slot%n] = b
			filled++
			continue
		}
		if prev == nil {
			return scale.BlockRootProof{}, fmt.Errorf("cannot reconstruct block roots: leading gap at slot %d (misses=%d)", slot, misses)
		}
		b := make([]byte, 32)
		copy(b, prev)
		blockRoots[slot%n] = b
	}

	for i := uint64(0); i < n; i++ {
		if blockRoots[i] == nil {
			return scale.BlockRootProof{}, fmt.Errorf("block roots ring incomplete at index %d", i)
		}
	}

	container := &state.BlockRootsContainerMainnet{}
	container.SetBlockRoots(blockRoots)
	tree, err := container.GetTree()
	if err != nil {
		return scale.BlockRootProof{}, fmt.Errorf("build block roots tree: %w", err)
	}
	leaf := tree.Hash()

	log.WithFields(log.Fields{
		"stateSlot":      stateSlot,
		"filledFromAPI":  filled,
		"forwardFilled":  int(n) - filled,
		"blockRootsRoot": fmt.Sprintf("0x%x", leaf),
	}).Info("reconstructed block_roots tree from headers")

	return scale.BlockRootProof{
		Leaf:  types.H256(common.BytesToHash(leaf)),
		Proof: nil,
		Tree:  tree,
	}, nil
}

func (s *Syncer) getSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error) {
	if s.stateService == nil {
		return nil, fmt.Errorf("state service is required but not configured")
	}

	var result *scale.SyncCommitteeProof
	err := s.retryWithBackoff(slot, "sync committee proof", false, func() error {
		proof, err := s.stateService.GetSyncCommitteeProof(slot, period)
		if err != nil {
			return err
		}
		log.WithFields(log.Fields{"slot": slot, "period": period}).Debug("got sync committee proof from state service")
		result = proof
		return nil
	})

	if err != nil {
		return nil, err
	}
	return result, nil
}

func (s *Syncer) getFinalizedHeaderProof(slot uint64) ([]types.H256, error) {
	if s.stateService == nil {
		return nil, fmt.Errorf("state service is required but not configured")
	}

	var result []types.H256
	err := s.retryWithBackoff(slot, "finalized header proof", false, func() error {
		proof, err := s.stateService.GetFinalizedHeaderProof(slot)
		if err != nil {
			return err
		}
		log.WithField("slot", slot).Debug("got finalized header proof from state service")
		result = proof
		return nil
	})

	if err != nil {
		return nil, err
	}
	return result, nil
}

// Abandon tip-following requests only within two epochs of finality (not historical slots).
func (s *Syncer) checkForNewerFinalizedSlot(requestedSlot uint64) (uint64, bool) {
	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		log.WithError(err).Debug("failed to check for newer finalized slot")
		return 0, false
	}

	currentFinalizedSlot, err := strconv.ParseUint(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		log.WithError(err).Debug("failed to parse current finalized slot")
		return 0, false
	}

	if currentFinalizedSlot <= requestedSlot {
		return 0, false
	}

	nearTipWindow := s.protocol.Settings.SlotsInEpoch * 2
	if currentFinalizedSlot-requestedSlot > nearTipWindow {
		return 0, false
	}

	return currentFinalizedSlot, true
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
	// Header-derived LC response lacks finality branch; build from state service instead.
	if finalizedUpdate.DerivedFromHeaders {
		if s.stateService == nil {
			return scale.Update{}, fmt.Errorf("node does not serve the light client API and no state service is configured")
		}
		finalizedSlot, err := util.ToUint64(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot)
		if err != nil {
			return scale.Update{}, fmt.Errorf("parse finalized slot: %w", err)
		}
		return s.findFinalizedUpdateAtOrBefore(finalizedSlot)
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
	maxSlotsMissed := int(s.protocol.Settings.SlotsInEpoch) * 2
	startSlot := slot
	for errors.Is(err, api.ErrNotFound) && tries < maxSlotsMissed {
		// GetBeaconBlockRoot returns the previous root on empty slots; use GetHeaderBySlot.
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

	header, err := s.Client.GetHeaderByBlockRoot(blockRoot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("fetch block: %w", err)
	}

	slot := header.Slot
	forkVersion := s.protocol.ForkVersion(slot)

	if forkVersion == protocol.Gloas {
		return s.getGloasHeaderUpdate(blockRoot, header, checkpoint)
	}

	blockBytes, err := s.Client.GetBeaconBlockBytes(blockRoot)
	if err != nil {
		return update, fmt.Errorf("fetch block: %w", err)
	}

	var signedBlock state.SignedBeaconBlock
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

func (s *Syncer) FindValidHeaderPair(minSlot, maxSlot uint64) (uint64, uint64, error) {
	var slot uint64
	if minSlot%s.protocol.Settings.SlotsInEpoch == 0 {
		slot = minSlot
	} else {
		slot = ((minSlot / s.protocol.Settings.SlotsInEpoch) + 1) * s.protocol.Settings.SlotsInEpoch
	}

	var lastErr error
	for {
		finalizedSlot, attestedSlot, err := s.findValidUpdatePair(slot)
		if err != nil {
			lastErr = err
			if slot >= maxSlot {
				return 0, 0, fmt.Errorf("unable to find valid slot pair in [%d, %d]: %w", minSlot, maxSlot, lastErr)
			}

			slot += s.protocol.Settings.SlotsInEpoch
			continue
		}

		log.WithFields(log.Fields{"attested": attestedSlot, "finalized": finalizedSlot}).Info("found boundary headers")
		return finalizedSlot, attestedSlot, nil
	}
}

func (s *Syncer) FindValidAttestedHeader(minSlot, maxSlot uint64) (uint64, error) {
	_, attestedSlot, err := s.FindValidHeaderPair(minSlot, maxSlot)
	return attestedSlot, err
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
	maxSlot := ((syncCommitteePeriod + 1) * s.protocol.SlotsPerHistoricalRoot) - s.protocol.Settings.SlotsInEpoch

	finalizedSlot, attestedSlot, err := s.FindValidHeaderPair(minSlot, maxSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries for period %d: %w", syncCommitteePeriod, err)
	}

	return s.buildFinalizedUpdateFromSlotPair(finalizedSlot, attestedSlot, true)
}

func (s *Syncer) GetFinalizedUpdateAtAttestedSlot(minSlot, maxSlot uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
	if s.stateService == nil {
		return scale.Update{}, fmt.Errorf("state service is required but not configured")
	}

	return s.getFinalizedUpdateFromStateService(minSlot, maxSlot, fetchNextSyncCommittee)
}

func (s *Syncer) getFinalizedUpdateFromStateService(minSlot, maxSlot uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
	attestedSlot, err := s.FindValidAttestedHeader(minSlot, maxSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("cannot find blocks at boundaries: %w", err)
	}

	finalizedUpdate, err := s.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return scale.Update{}, fmt.Errorf("get finalized update from API: %w", err)
	}

	finalizedSlot, err := util.ToUint64(finalizedUpdate.Data.FinalizedHeader.Beacon.Slot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("parse finalized slot: %w", err)
	}

	log.WithFields(log.Fields{"finalizedSlot": finalizedSlot, "attestedSlot": attestedSlot}).Info("found slot pair for finalized update")

	return s.buildFinalizedUpdateFromSlotPair(finalizedSlot, attestedSlot, fetchNextSyncCommittee)
}

func (s *Syncer) buildFinalizedUpdateFromSlotPair(finalizedSlot, attestedSlot uint64, fetchNextSyncCommittee bool) (scale.Update, error) {
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

	header, err := s.Client.GetHeaderBySlot(attestedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch header at slot: %w", err)
	}

	finalizedHeader, err := s.Client.GetHeaderBySlot(finalizedSlot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch finalized header at slot: %w", err)
	}

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

// Finalized header must be FFG checkpoint root, not headers/finalized (epoch-aligned fallback if FFG API fails).
func (s *Syncer) findFinalizedUpdateAtOrBefore(finalizedSlot uint64) (scale.Update, error) {
	if update, err := s.findFinalizedUpdateFromFFGCheckpoint(); err == nil {
		return update, nil
	} else {
		log.WithError(err).WithField("finalizedSlot", finalizedSlot).
			Debug("FFG checkpoint update unavailable, falling back to epoch-aligned search")
	}

	epoch := s.protocol.Settings.SlotsInEpoch
	slot := finalizedSlot - finalizedSlot%epoch

	var lastErr error
	for attempt := 0; attempt < maxFinalizedUpdateLookback; attempt++ {
		pairFinalized, pairAttested, err := s.findValidUpdatePair(slot)
		if err == nil {
			log.WithFields(log.Fields{"finalized": pairFinalized, "attested": pairAttested}).
				Info("built finalized update from the state service")
			return s.buildFinalizedUpdateFromSlotPair(pairFinalized, pairAttested, true)
		}
		lastErr = err
		if slot < epoch {
			break
		}
		slot -= epoch
	}
	return scale.Update{}, fmt.Errorf("no valid slot pair within %d epochs of slot %d: %w", maxFinalizedUpdateLookback, finalizedSlot, lastErr)
}

func (s *Syncer) findFinalizedUpdateFromFFGCheckpoint() (scale.Update, error) {
	epoch := s.protocol.Settings.SlotsInEpoch

	checkpoint, err := s.Client.GetFinalizedCheckpoint()
	if err != nil {
		return scale.Update{}, fmt.Errorf("get finalized checkpoint: %w", err)
	}
	if checkpoint.Epoch == 0 && checkpoint.FinalizedBlockRoot == (common.Hash{}) {
		return scale.Update{}, fmt.Errorf("empty finalized checkpoint")
	}

	checkpointHeader, err := s.Client.GetHeaderByBlockRoot(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return scale.Update{}, fmt.Errorf("get checkpoint header: %w", err)
	}
	finalizedSlot := checkpointHeader.Slot

	slot := (checkpoint.Epoch + 2) * epoch
	maxSlot := slot + epoch*uint64(maxFinalizedUpdateLookback)
	if head, headErr := s.Client.GetHeaderAtHead(); headErr == nil && head.Slot > 0 && head.Slot < maxSlot {
		maxSlot = head.Slot
	}
	var lastErr error
	for slot <= maxSlot {
		attestedHeader, err := s.FindBeaconHeaderWithBlockIncluded(slot)
		if err != nil {
			lastErr = err
			if slot+epoch > maxSlot {
				break
			}
			slot += epoch
			continue
		}
		attestedSlot := attestedHeader.Slot

		nextHeader, err := s.FindBeaconHeaderWithBlockIncluded(attestedSlot + 1)
		if err != nil {
			lastErr = err
			if attestedSlot >= maxSlot {
				break
			}
			slot = attestedSlot + 1
			continue
		}
		nextBlock, err := s.Client.GetBeaconBlockBySlot(nextHeader.Slot)
		if err != nil {
			lastErr = err
			slot = attestedSlot + 1
			continue
		}
		superMajority, err := s.protocol.SyncCommitteeSuperMajority(nextBlock.Data.Message.Body.SyncAggregate.SyncCommitteeBits)
		if err != nil {
			lastErr = err
			slot = attestedSlot + 1
			continue
		}
		if !superMajority {
			lastErr = fmt.Errorf("sync committee at slot not supermajority: %d", nextHeader.Slot)
			slot = attestedSlot + 1
			continue
		}

		if s.stateService == nil {
			return scale.Update{}, fmt.Errorf("state service is required but not configured")
		}
		leaf, err := s.stateService.GetFinalizedHeaderCheckpointRoot(attestedSlot)
		if err != nil {
			lastErr = err
			slot = attestedSlot + 1
			continue
		}
		if leaf != checkpoint.FinalizedBlockRoot {
			lastErr = fmt.Errorf(
				"attested slot %d proves checkpoint %s, want %s",
				attestedSlot,
				leaf.Hex(),
				checkpoint.FinalizedBlockRoot.Hex(),
			)
			slot = attestedSlot + 1
			continue
		}

		log.WithFields(log.Fields{
			"finalized": finalizedSlot,
			"attested":  attestedSlot,
			"epoch":     checkpoint.Epoch,
			"root":      checkpoint.FinalizedBlockRoot.Hex(),
		}).Info("built finalized update from FFG checkpoint")
		return s.buildFinalizedUpdateFromSlotPair(finalizedSlot, attestedSlot, true)
	}

	return scale.Update{}, fmt.Errorf(
		"no attested header proving checkpoint within %d epochs of slot %d: %w",
		maxFinalizedUpdateLookback,
		(checkpoint.Epoch+2)*epoch,
		lastErr,
	)
}

const maxFinalizedUpdateLookback = 16
