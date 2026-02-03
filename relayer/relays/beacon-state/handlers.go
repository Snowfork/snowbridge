package beaconstate

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type HealthResponse struct {
	Status         string `json:"status"`
	ProofCacheSize int    `json:"proofCacheSize"`
	BeaconEndpoint string `json:"beaconEndpoint"`
}

type ProofResponse struct {
	Slot             uint64   `json:"slot"`
	Leaf             string   `json:"leaf"`
	Proof            []string `json:"proof"`
	GeneralizedIndex int      `json:"generalizedIndex"`
}

type BlockRootProofResponse struct {
	Slot             uint64   `json:"slot"`
	Leaf             string   `json:"leaf"`
	Proof            []string `json:"proof"`
	GeneralizedIndex int      `json:"generalizedIndex"`
	BlockRoots       []string `json:"blockRoots"`
}

type SyncCommitteeProofResponse struct {
	Slot             uint64   `json:"slot"`
	Leaf             string   `json:"leaf"`
	Proof            []string `json:"proof"`
	GeneralizedIndex int      `json:"generalizedIndex"`
	Pubkeys          []string `json:"pubkeys"`
	AggregatePubkey  string   `json:"aggregatePubkey"`
}

type ErrorResponse struct {
	Error string `json:"error"`
}

func (s *Service) handleHealth(w http.ResponseWriter, r *http.Request) {
	response := HealthResponse{
		Status:         "healthy",
		ProofCacheSize: s.proofCache.Size(),
		BeaconEndpoint: s.config.Beacon.Endpoint,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func (s *Service) handleFinalizedHeaderProof(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	cacheKey := fmt.Sprintf("finalized-header:%d", slot)

	// Only return from cache - finality watcher handles all proof generation
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithField("slot", slot).Debug("Returning cached finalized header proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Proof not cached - return 503 to signal retry
	log.WithField("slot", slot).Debug("Proof not cached, returning 503 for client retry")
	w.Header().Set("Retry-After", "5")
	writeError(w, http.StatusServiceUnavailable, "proof not ready, please retry")
}

func (s *Service) handleBlockRootProof(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	cacheKey := fmt.Sprintf("block-root:%d", slot)

	// Only return from cache - finality watcher handles all proof generation
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithField("slot", slot).Debug("Returning cached block root proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Proof not cached - return 503 to signal retry
	// Finality watcher will pre-generate proofs when state is downloaded
	log.WithField("slot", slot).Debug("Proof not cached, returning 503 for client retry")
	w.Header().Set("Retry-After", "5")
	writeError(w, http.StatusServiceUnavailable, "proof not ready, please retry")
}

func (s *Service) handleSyncCommitteeProof(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	period := r.URL.Query().Get("period")
	if period == "" {
		period = "current"
	}

	cacheKey := fmt.Sprintf("sync-committee:%d:%s", slot, period)

	// Only return from cache - finality watcher handles all proof generation
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithFields(log.Fields{"slot": slot, "period": period}).Debug("Returning cached sync committee proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Proof not cached - return 503 to signal retry
	log.WithFields(log.Fields{"slot": slot, "period": period}).Debug("Proof not cached, returning 503 for client retry")
	w.Header().Set("Retry-After", "5")
	writeError(w, http.StatusServiceUnavailable, "proof not ready, please retry")
}

// cacheAllProofs generates all proof types for a slot and caches them
func (s *Service) cacheAllProofs(slot uint64, beaconState state.BeaconState, tree *ssz.Node) {
	// 1. Finalized header proof
	s.cacheProof(slot, "finalized-header", s.protocol.FinalizedCheckpointGeneralizedIndex(slot), tree)

	// 2. Block root proof (includes block roots array)
	s.cacheBlockRootProof(slot, beaconState, tree)

	// 3. Sync committee proofs (current and next) - includes pubkeys
	s.cacheSyncCommitteeProof(slot, "current", beaconState.GetCurrentSyncCommittee(), s.protocol.CurrentSyncCommitteeGeneralizedIndex(slot), tree)
	s.cacheSyncCommitteeProof(slot, "next", beaconState.GetNextSyncCommittee(), s.protocol.NextSyncCommitteeGeneralizedIndex(slot), tree)

	log.WithField("slot", slot).Info("Cached all proofs for slot")
}

func (s *Service) cacheProof(slot uint64, proofType string, generalizedIndex int, tree *ssz.Node) {
	proof, err := tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).WithFields(log.Fields{"slot": slot, "proofType": proofType}).Warn("Failed to generate proof")
		return
	}

	cacheKey := fmt.Sprintf("%s:%d", proofType, slot)

	response := ProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
	}

	jsonResponse, err := json.Marshal(response)
	if err != nil {
		log.WithError(err).WithFields(log.Fields{"slot": slot, "proofType": proofType}).Warn("Failed to marshal proof response")
		return
	}
	s.proofCache.Put(cacheKey, jsonResponse)
}

func (s *Service) cacheSyncCommitteeProof(slot uint64, period string, syncCommittee *state.SyncCommittee, generalizedIndex int, tree *ssz.Node) {
	proof, err := tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).WithFields(log.Fields{"slot": slot, "period": period}).Warn("Failed to generate sync committee proof")
		return
	}

	// Convert pubkeys to hex strings
	pubkeysHex := make([]string, len(syncCommittee.PubKeys))
	for i, pk := range syncCommittee.PubKeys {
		pubkeysHex[i] = "0x" + hex.EncodeToString(pk)
	}

	cacheKey := fmt.Sprintf("sync-committee:%d:%s", slot, period)

	response := SyncCommitteeProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
		Pubkeys:          pubkeysHex,
		AggregatePubkey:  "0x" + hex.EncodeToString(syncCommittee.AggregatePubKey[:]),
	}

	jsonResponse, err := json.Marshal(response)
	if err != nil {
		log.WithError(err).WithFields(log.Fields{"slot": slot, "period": period}).Warn("Failed to marshal sync committee proof response")
		return
	}
	s.proofCache.Put(cacheKey, jsonResponse)
}

func (s *Service) cacheBlockRootProof(slot uint64, beaconState state.BeaconState, tree *ssz.Node) {
	generalizedIndex := s.protocol.BlockRootGeneralizedIndex(slot)
	proof, err := tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Warn("Failed to generate block root proof")
		return
	}

	// Get block roots from state
	blockRoots := beaconState.GetBlockRoots()
	blockRootsHex := make([]string, len(blockRoots))
	for i, root := range blockRoots {
		blockRootsHex[i] = "0x" + hex.EncodeToString(root[:])
	}

	response := BlockRootProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
		BlockRoots:       blockRootsHex,
	}

	cacheKey := fmt.Sprintf("block-root:%d", slot)
	jsonResponse, err := json.Marshal(response)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Warn("Failed to marshal block root proof response")
		return
	}
	s.proofCache.Put(cacheKey, jsonResponse)
}

// hasAllProofsCached checks if at least one proof is cached for the slot.
// Used for double-check after acquiring the download lock.
func (s *Service) hasAllProofsCached(slot uint64) bool {
	// Just check one proof type - if one is cached, all should be cached
	cacheKey := fmt.Sprintf("finalized-header:%d", slot)
	_, ok := s.proofCache.Get(cacheKey)
	return ok
}

func (s *Service) unmarshalBeaconState(slot uint64, data []byte) (state.BeaconState, error) {
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
		return nil, err
	}

	return beaconState, nil
}

// unmarshalBeaconStateLite unmarshals beacon state using the memory-efficient lite parser.
// This saves ~130MB+ by only extracting fields needed for proof generation and computing
// hashes for the rest without storing the raw data.
func (s *Service) unmarshalBeaconStateLite(slot uint64, data []byte) (state.BeaconState, error) {
	forkVersion := s.protocol.ForkVersion(slot)
	log.WithFields(log.Fields{"slot": slot, "forkVersion": forkVersion, "dataSize": len(data)}).Info("Unmarshaling beacon state with lite parser")

	if forkVersion == protocol.Fulu {
		// Parse with lite parser
		liteState, err := UnmarshalSSZLiteFulu(data)
		if err != nil {
			return nil, fmt.Errorf("unmarshal lite fulu state: %w", err)
		}

		// DEBUG: Compare with full parser to verify tree roots match
		fullState := &state.BeaconStateFulu{}
		if err := fullState.UnmarshalSSZ(data); err != nil {
			log.WithError(err).Warn("DEBUG: Failed to unmarshal full state for comparison")
		} else {
			fullRoot, err1 := fullState.HashTreeRoot()
			liteRoot, err2 := liteState.HashTreeRoot()
			if err1 != nil || err2 != nil {
				log.WithFields(log.Fields{"fullErr": err1, "liteErr": err2}).Warn("DEBUG: Failed to compute tree roots")
			} else {
				if fullRoot != liteRoot {
					log.WithFields(log.Fields{
						"fullRoot": fmt.Sprintf("0x%x", fullRoot),
						"liteRoot": fmt.Sprintf("0x%x", liteRoot),
					}).Error("DEBUG: Tree root MISMATCH between full and lite state!")

					// Compare individual field trees to find the mismatch
					fullTree, _ := fullState.GetTree()
					liteTree, _ := liteState.GetTree()
					if fullTree != nil && liteTree != nil {
						for fieldIdx := 0; fieldIdx < 40; fieldIdx++ {
							gidx := 64 + fieldIdx // Generalized index for fields in a 64-leaf tree
							fullProof, err1 := fullTree.Prove(gidx)
							liteProof, err2 := liteTree.Prove(gidx)
							if err1 != nil || err2 != nil {
								continue
							}
							if string(fullProof.Leaf) != string(liteProof.Leaf) {
								log.WithFields(log.Fields{
									"field":    fieldIdx,
									"fullLeaf": fmt.Sprintf("0x%x", fullProof.Leaf),
									"liteLeaf": fmt.Sprintf("0x%x", liteProof.Leaf),
								}).Warn("DEBUG: Field mismatch")
							}
						}
					}
				} else {
					log.WithField("root", fmt.Sprintf("0x%x", fullRoot)).Info("DEBUG: Tree roots match")
				}
			}
		}

		return liteState, nil
	}

	if forkVersion == protocol.Electra {
		liteState, err := UnmarshalSSZLiteElectra(data)
		if err != nil {
			return nil, fmt.Errorf("unmarshal lite electra state: %w", err)
		}
		return liteState, nil
	}

	// Deneb
	liteState, err := UnmarshalSSZLiteDeneb(data)
	if err != nil {
		return nil, fmt.Errorf("unmarshal lite deneb state: %w", err)
	}
	return liteState, nil
}

func parseSlotParam(r *http.Request) (uint64, error) {
	slotStr := r.URL.Query().Get("slot")
	if slotStr == "" {
		return 0, fmt.Errorf("slot parameter is required")
	}
	slot, err := strconv.ParseUint(slotStr, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid slot: %w", err)
	}
	return slot, nil
}

func hashesToHexStrings(hashes [][]byte) []string {
	result := make([]string, len(hashes))
	for i, hash := range hashes {
		result[i] = "0x" + hex.EncodeToString(hash)
	}
	return result
}

func writeError(w http.ResponseWriter, status int, message string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(ErrorResponse{Error: message})
}
