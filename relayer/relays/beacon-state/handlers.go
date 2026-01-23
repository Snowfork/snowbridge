package beaconstate

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"runtime"
	"strconv"

	log "github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type HealthResponse struct {
	Status            string `json:"status"`
	LatestCachedSlot  uint64 `json:"latestCachedSlot,omitempty"`
	StateCacheSize    int    `json:"stateCacheSize"`
	ProofCacheSize    int    `json:"proofCacheSize"`
	BeaconEndpoint    string `json:"beaconEndpoint"`
}

type ProofResponse struct {
	Slot             uint64   `json:"slot"`
	Leaf             string   `json:"leaf"`
	Proof            []string `json:"proof"`
	GeneralizedIndex int      `json:"generalizedIndex"`
}

type BlockRootProofResponse struct {
	Slot             uint64     `json:"slot"`
	Leaf             string     `json:"leaf"`
	Proof            []string   `json:"proof"`
	GeneralizedIndex int        `json:"generalizedIndex"`
	BlockRoots       []string   `json:"blockRoots"`
}

type ErrorResponse struct {
	Error string `json:"error"`
}

func (s *Service) handleHealth(w http.ResponseWriter, r *http.Request) {
	response := HealthResponse{
		Status:         "healthy",
		StateCacheSize: s.stateCache.Size(),
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

	// Check proof cache
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithField("slot", slot).Debug("Returning cached finalized header proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Get or download state
	cachedState, err := s.getOrDownloadState(slot)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Error("Failed to get beacon state")
		writeError(w, http.StatusInternalServerError, "failed to get beacon state")
		return
	}

	// Generate proof
	_ = cachedState.Tree.Hash()
	generalizedIndex := s.protocol.FinalizedCheckpointGeneralizedIndex(slot)
	proof, err := cachedState.Tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).Error("Failed to generate finalized header proof")
		writeError(w, http.StatusInternalServerError, "failed to generate proof")
		return
	}

	response := ProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
	}

	// Cache and return
	jsonResponse, _ := json.Marshal(response)
	s.proofCache.Put(cacheKey, jsonResponse)

	w.Header().Set("Content-Type", "application/json")
	w.Write(jsonResponse)
}

func (s *Service) handleExecutionStateRootProof(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	cacheKey := fmt.Sprintf("execution-state-root:%d", slot)

	// Check proof cache
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithField("slot", slot).Debug("Returning cached execution state root proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Get or download state
	cachedState, err := s.getOrDownloadState(slot)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Error("Failed to get beacon state")
		writeError(w, http.StatusInternalServerError, "failed to get beacon state")
		return
	}

	// Generate proof for execution payload
	_ = cachedState.Tree.Hash()
	generalizedIndex := s.protocol.ExecutionPayloadGeneralizedIndex(slot)
	proof, err := cachedState.Tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).Error("Failed to generate execution state root proof")
		writeError(w, http.StatusInternalServerError, "failed to generate proof")
		return
	}

	response := ProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
	}

	// Cache and return
	jsonResponse, _ := json.Marshal(response)
	s.proofCache.Put(cacheKey, jsonResponse)

	w.Header().Set("Content-Type", "application/json")
	w.Write(jsonResponse)
}

func (s *Service) handleBlockRootProof(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	cacheKey := fmt.Sprintf("block-root:%d", slot)

	// Check proof cache
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithField("slot", slot).Debug("Returning cached block root proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Get or download state
	cachedState, err := s.getOrDownloadState(slot)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Error("Failed to get beacon state")
		writeError(w, http.StatusInternalServerError, "failed to get beacon state")
		return
	}

	// Generate proof for block roots
	_ = cachedState.Tree.Hash()
	generalizedIndex := s.protocol.BlockRootGeneralizedIndex(slot)
	proof, err := cachedState.Tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).Error("Failed to generate block root proof")
		writeError(w, http.StatusInternalServerError, "failed to generate proof")
		return
	}

	// Get block roots from state for ancestry proofs
	blockRoots := cachedState.State.GetBlockRoots()
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

	// Cache and return
	jsonResponse, _ := json.Marshal(response)
	s.proofCache.Put(cacheKey, jsonResponse)

	w.Header().Set("Content-Type", "application/json")
	w.Write(jsonResponse)
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

	// Check proof cache
	if cached, ok := s.proofCache.Get(cacheKey); ok {
		log.WithFields(log.Fields{"slot": slot, "period": period}).Debug("Returning cached sync committee proof")
		w.Header().Set("Content-Type", "application/json")
		w.Write(cached)
		return
	}

	// Get or download state
	cachedState, err := s.getOrDownloadState(slot)
	if err != nil {
		log.WithError(err).WithField("slot", slot).Error("Failed to get beacon state")
		writeError(w, http.StatusInternalServerError, "failed to get beacon state")
		return
	}

	// Generate proof for sync committee
	_ = cachedState.Tree.Hash()
	var generalizedIndex int
	if period == "next" {
		generalizedIndex = s.protocol.NextSyncCommitteeGeneralizedIndex(slot)
	} else {
		generalizedIndex = s.protocol.CurrentSyncCommitteeGeneralizedIndex(slot)
	}

	proof, err := cachedState.Tree.Prove(generalizedIndex)
	if err != nil {
		log.WithError(err).Error("Failed to generate sync committee proof")
		writeError(w, http.StatusInternalServerError, "failed to generate proof")
		return
	}

	response := ProofResponse{
		Slot:             slot,
		Leaf:             "0x" + hex.EncodeToString(proof.Leaf),
		Proof:            hashesToHexStrings(proof.Hashes),
		GeneralizedIndex: generalizedIndex,
	}

	// Cache and return
	jsonResponse, _ := json.Marshal(response)
	s.proofCache.Put(cacheKey, jsonResponse)

	w.Header().Set("Content-Type", "application/json")
	w.Write(jsonResponse)
}

func (s *Service) getOrDownloadState(slot uint64) (*CachedState, error) {
	// Check state cache
	if cached, ok := s.stateCache.Get(slot); ok {
		log.WithField("slot", slot).Debug("Using cached beacon state")
		return cached, nil
	}

	log.WithField("slot", slot).Info("Downloading beacon state")

	// Download state
	data, err := s.syncer.Client.GetBeaconState(strconv.FormatUint(slot, 10))
	if err != nil {
		return nil, fmt.Errorf("download beacon state: %w", err)
	}

	// Unmarshal based on fork
	beaconState, err := s.unmarshalBeaconState(slot, data)
	if err != nil {
		return nil, fmt.Errorf("unmarshal beacon state: %w", err)
	}

	// Generate tree
	tree, err := beaconState.GetTree()
	if err != nil {
		return nil, fmt.Errorf("get state tree: %w", err)
	}

	// Cache
	s.stateCache.Put(slot, beaconState, tree)

	// Trigger GC to help reclaim memory
	runtime.GC()

	cached, _ := s.stateCache.Get(slot)
	return cached, nil
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

// StateResponse is the response for raw beacon state requests
type StateResponse struct {
	Slot uint64 `json:"slot"`
	Data string `json:"data"` // hex-encoded SSZ state data
}

// StateRangeResponse is the response for beacon state range requests
type StateRangeResponse struct {
	AttestedSlot  uint64 `json:"attestedSlot"`
	FinalizedSlot uint64 `json:"finalizedSlot"`
	AttestedData  string `json:"attestedData"`  // hex-encoded SSZ state data
	FinalizedData string `json:"finalizedData"` // hex-encoded SSZ state data
}

// handleGetState returns raw beacon state data for a given slot
// First checks persistent store, then fetches from beacon node
func (s *Service) handleGetState(w http.ResponseWriter, r *http.Request) {
	slot, err := parseSlotParam(r)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	log.WithField("slot", slot).Debug("Handling get state request")

	// Try persistent store first
	data, err := s.store.GetBeaconStateData(slot)
	if err == nil {
		log.WithField("slot", slot).Debug("Found state in persistent store")
		response := StateResponse{
			Slot: slot,
			Data: "0x" + hex.EncodeToString(data),
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
		return
	}

	// Fetch from beacon node
	data, err = s.syncer.Client.GetBeaconState(strconv.FormatUint(slot, 10))
	if err != nil {
		log.WithError(err).WithField("slot", slot).Error("Failed to get beacon state")
		writeError(w, http.StatusNotFound, "beacon state not found")
		return
	}

	response := StateResponse{
		Slot: slot,
		Data: "0x" + hex.EncodeToString(data),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

// handleGetStateInRange finds beacon states within a slot range from the persistent store
// Query params: minSlot, maxSlot
func (s *Service) handleGetStateInRange(w http.ResponseWriter, r *http.Request) {
	minSlotStr := r.URL.Query().Get("minSlot")
	maxSlotStr := r.URL.Query().Get("maxSlot")

	if minSlotStr == "" || maxSlotStr == "" {
		writeError(w, http.StatusBadRequest, "minSlot and maxSlot parameters are required")
		return
	}

	minSlot, err := strconv.ParseUint(minSlotStr, 10, 64)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid minSlot")
		return
	}

	maxSlot, err := strconv.ParseUint(maxSlotStr, 10, 64)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid maxSlot")
		return
	}

	log.WithFields(log.Fields{"minSlot": minSlot, "maxSlot": maxSlot}).Debug("Handling get state in range request")

	// Query persistent store
	data, err := s.store.FindBeaconStateWithinRange(minSlot, maxSlot)
	if err != nil {
		log.WithError(err).WithFields(log.Fields{"minSlot": minSlot, "maxSlot": maxSlot}).Debug("No states found in range")
		writeError(w, http.StatusNotFound, "no beacon states found in range")
		return
	}

	response := StateRangeResponse{
		AttestedSlot:  data.AttestedSlot,
		FinalizedSlot: data.FinalizedSlot,
		AttestedData:  "0x" + hex.EncodeToString(data.AttestedBeaconState),
		FinalizedData: "0x" + hex.EncodeToString(data.FinalizedBeaconState),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}
