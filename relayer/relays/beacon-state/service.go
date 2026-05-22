package beaconstate

import (
	"context"
	"fmt"
	"net/http"
	"runtime"
	"strconv"
	"sync"
	"time"

	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
)

type Service struct {
	config              *Config
	syncer              *syncer.Syncer
	protocol            *protocol.Protocol
	store               *store.Store
	proofCache          *ProofCache
	httpServer          *http.Server
	downloadMu          sync.Mutex  // Ensures only one state download at a time
	lastFinalizedSlot   uint64      // Tracks the last seen finalized slot for the watcher
	slotMu              sync.Mutex  // Protects lastFinalizedSlot
	watcherDownloading  bool        // True if watcher is currently downloading
	watcherDownloadSlot uint64      // The slot currently being downloaded by watcher
}

func New(config *Config) *Service {
	return &Service{
		config: config,
	}
}

func (s *Service) Start(ctx context.Context, eg *errgroup.Group) error {
	specSettings := s.config.Beacon.Spec
	log.WithField("spec", specSettings).Info("spec settings")

	// Initialize protocol
	// HeaderRedundancy is not used in state service, set to 0
	s.protocol = protocol.New(specSettings, 0)

	// Initialize store
	// Use persist.maxEntries if persist is enabled, otherwise fall back to beacon.datastore.maxEntries
	maxEntries := s.config.Beacon.DataStore.MaxEntries
	if s.config.Persist.Enabled && s.config.Persist.MaxEntries > 0 {
		maxEntries = s.config.Persist.MaxEntries
	}
	st := store.New(s.config.Beacon.DataStore.Location, maxEntries, *s.protocol)
	err := st.Connect()
	if err != nil {
		return fmt.Errorf("connect to store: %w", err)
	}
	s.store = &st

	// Initialize beacon API client
	beaconAPI := api.NewBeaconClient(s.config.Beacon.Endpoint)

	// Initialize syncer without state service (this IS the state service)
	// The syncer will fall back to beacon API directly
	s.syncer = syncer.New(beaconAPI, s.protocol, nil)

	// Initialize proof cache
	proofTTL := time.Duration(s.config.Cache.ProofTTLSeconds) * time.Second
	s.proofCache = NewProofCache(s.config.Cache.MaxProofs, proofTTL)

	// Parse timeouts with fallback defaults
	readTimeout, err := time.ParseDuration(s.config.HTTP.ReadTimeout)
	if err != nil {
		log.WithError(err).WithField("value", s.config.HTTP.ReadTimeout).Warn("Failed to parse HTTP read timeout, using default 30s")
		readTimeout = 30 * time.Second
	}
	writeTimeout, err := time.ParseDuration(s.config.HTTP.WriteTimeout)
	if err != nil {
		log.WithError(err).WithField("value", s.config.HTTP.WriteTimeout).Warn("Failed to parse HTTP write timeout, using default 30s")
		writeTimeout = 30 * time.Second
	}

	// Setup HTTP server
	mux := http.NewServeMux()
	mux.HandleFunc("/health", s.handleHealth)
	mux.HandleFunc("/v1/proofs/finalized-header", s.handleFinalizedHeaderProof)
	mux.HandleFunc("/v1/proofs/block-root", s.handleBlockRootProof)
	mux.HandleFunc("/v1/proofs/sync-committee", s.handleSyncCommitteeProof)

	s.httpServer = &http.Server{
		Addr:         fmt.Sprintf(":%d", s.config.HTTP.Port),
		Handler:      mux,
		ReadTimeout:  readTimeout,
		WriteTimeout: writeTimeout,
	}

	// Start HTTP server FIRST so health checks pass immediately
	eg.Go(func() error {
		log.WithField("port", s.config.HTTP.Port).Info("Starting beacon state service HTTP server")
		err := s.httpServer.ListenAndServe()
		if err != nil && err != http.ErrServerClosed {
			return fmt.Errorf("http server: %w", err)
		}
		return nil
	})

	// Do initial state download in background after HTTP server starts
	// This ensures the service is healthy immediately while states are being cached
	if s.config.Watch.Enabled {
		eg.Go(func() error {
			// Small delay to ensure HTTP server is up
			time.Sleep(100 * time.Millisecond)
			log.Info("Downloading initial finalized beacon states in background...")
			if err := s.downloadCurrentFinalizedStateSync(); err != nil {
				log.WithError(err).Warn("Failed to download initial beacon states, will retry via finality watcher")
			} else {
				log.Info("Initial beacon states downloaded successfully")
			}
			return nil
		})
	}

	// Graceful shutdown
	eg.Go(func() error {
		<-ctx.Done()
		log.Info("Shutting down beacon state service HTTP server")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return s.httpServer.Shutdown(shutdownCtx)
	})

	// Start periodic state saving if enabled
	if s.config.Persist.Enabled {
		eg.Go(func() error {
			return s.runPeriodicStateSaver(ctx)
		})
	}

	// Start finality watcher if enabled
	if s.config.Watch.Enabled {
		eg.Go(func() error {
			return s.runFinalityWatcher(ctx)
		})
	}

	return nil
}

// runPeriodicStateSaver periodically fetches and saves beacon states to disk
func (s *Service) runPeriodicStateSaver(ctx context.Context) error {
	interval := time.Duration(s.config.Persist.SaveIntervalHours) * time.Hour
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	log.WithField("interval", interval).Info("Starting periodic beacon state saver")

	// Only save on startup if we don't have a recent state
	if s.shouldSaveOnStartup(interval) {
		if err := s.saveCurrentFinalizedState(); err != nil {
			log.WithError(err).Warn("Failed to save initial beacon state")
		}
	}

	for {
		select {
		case <-ctx.Done():
			log.Info("Stopping periodic beacon state saver")
			return nil
		case <-ticker.C:
			if err := s.saveCurrentFinalizedState(); err != nil {
				log.WithError(err).Warn("Failed to save beacon state")
			}
		}
	}
}

// shouldSaveOnStartup checks if we need to save a beacon state on startup.
// Returns true if no recent state exists (within the save interval).
func (s *Service) shouldSaveOnStartup(interval time.Duration) bool {
	latestTimestamp, err := s.store.GetLatestTimestamp()
	if err != nil {
		log.WithError(err).Warn("Failed to get latest beacon state timestamp, will save on startup")
		return true
	}

	// No entries exist
	if latestTimestamp.IsZero() {
		log.Info("No existing beacon states found, will save on startup")
		return true
	}

	// Check if the latest entry is older than the save interval
	age := time.Since(latestTimestamp)
	if age >= interval {
		log.WithFields(log.Fields{
			"lastSaved": latestTimestamp,
			"age":       age,
			"interval":  interval,
		}).Info("Latest beacon state is older than save interval, will save on startup")
		return true
	}

	log.WithFields(log.Fields{
		"lastSaved":    latestTimestamp,
		"age":          age,
		"nextSaveIn":   interval - age,
	}).Info("Recent beacon state exists, skipping startup save")
	return false
}

// saveCurrentFinalizedState fetches and saves the current finalized beacon state
func (s *Service) saveCurrentFinalizedState() error {
	log.Info("Fetching and saving current finalized beacon state")

	// Get the latest finalized update to find attested and finalized slots
	update, err := s.syncer.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("get finalized update: %w", err)
	}

	attestedSlot, err := strconv.ParseUint(update.Data.AttestedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse attested slot: %w", err)
	}
	finalizedSlot, err := strconv.ParseUint(update.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse finalized slot: %w", err)
	}

	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
	}).Info("Downloading beacon states")

	// Serialize beacon state downloads to prevent OOM from concurrent large state downloads
	s.downloadMu.Lock()
	defer s.downloadMu.Unlock()

	// Download attested state
	attestedData, err := s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", attestedSlot))
	if err != nil {
		return fmt.Errorf("download attested state at slot %d: %w", attestedSlot, err)
	}

	// Download finalized state
	finalizedData, err := s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", finalizedSlot))
	if err != nil {
		return fmt.Errorf("download finalized state at slot %d: %w", finalizedSlot, err)
	}

	// Write to store
	err = s.store.WriteEntry(attestedSlot, finalizedSlot, attestedData, finalizedData)
	if err != nil {
		return fmt.Errorf("write states to store: %w", err)
	}

	// Prune old states
	deletedSlots, err := s.store.PruneOldStates()
	if err != nil {
		log.WithError(err).Warn("Failed to prune old states")
	} else if len(deletedSlots) > 0 {
		log.WithField("deletedSlots", deletedSlots).Info("Pruned old beacon states")
	}

	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
	}).Info("Successfully saved beacon states")

	return nil
}

func (s *Service) GetSyncer() *syncer.Syncer {
	return s.syncer
}

// checkForNewerFinalizedSlot checks if a newer finalized slot exists than the one being processed.
// Returns the newer slot if one exists, otherwise returns 0.
func (s *Service) checkForNewerFinalizedSlot(currentSlot uint64) uint64 {
	update, err := s.syncer.Client.GetLatestFinalizedUpdate()
	if err != nil {
		log.WithError(err).Debug("Failed to check for newer finalized slot")
		return 0
	}

	latestFinalizedSlot, err := strconv.ParseUint(update.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		log.WithError(err).Debug("Failed to parse latest finalized slot")
		return 0
	}

	if latestFinalizedSlot > currentSlot {
		return latestFinalizedSlot
	}

	return 0
}

func (s *Service) GetProtocol() *protocol.Protocol {
	return s.protocol
}

func (s *Service) GetProofCache() *ProofCache {
	return s.proofCache
}

// runFinalityWatcher polls for new finalized blocks and pre-downloads beacon states
func (s *Service) runFinalityWatcher(ctx context.Context) error {
	interval := time.Duration(s.config.Watch.PollIntervalSeconds) * time.Second
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	log.WithField("interval", interval).Info("Starting finality watcher")

	// Do an initial check on startup
	if err := s.checkAndDownloadFinalizedState(ctx); err != nil {
		log.WithError(err).Warn("Initial finality check failed")
	}

	for {
		select {
		case <-ctx.Done():
			log.Info("Stopping finality watcher")
			return nil
		case <-ticker.C:
			if err := s.checkAndDownloadFinalizedState(ctx); err != nil {
				log.WithError(err).Warn("Finality check failed")
			}
		}
	}
}

// checkAndDownloadFinalizedState checks for new finalized blocks and pre-downloads states
func (s *Service) checkAndDownloadFinalizedState(ctx context.Context) error {
	// Get the latest finalized update
	update, err := s.syncer.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("get finalized update: %w", err)
	}

	attestedSlot, err := strconv.ParseUint(update.Data.AttestedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse attested slot: %w", err)
	}
	finalizedSlot, err := strconv.ParseUint(update.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse finalized slot: %w", err)
	}

	// Check if this is a new finalized slot or if we're already downloading it
	s.slotMu.Lock()
	lastSeen := s.lastFinalizedSlot
	alreadyDownloading := s.watcherDownloading && s.watcherDownloadSlot == finalizedSlot
	s.slotMu.Unlock()

	if finalizedSlot <= lastSeen {
		log.WithFields(log.Fields{
			"finalizedSlot": finalizedSlot,
			"lastSeen":      lastSeen,
		}).Debug("No new finalized block")
		return nil
	}

	if alreadyDownloading {
		log.WithField("finalizedSlot", finalizedSlot).Debug("Already downloading this finalized block")
		return nil
	}

	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
		"lastSeen":      lastSeen,
	}).Info("New finalized block detected, pre-downloading beacon states")

	// Mark as downloading
	s.slotMu.Lock()
	s.watcherDownloading = true
	s.watcherDownloadSlot = finalizedSlot
	s.slotMu.Unlock()

	// Download the states in a separate goroutine to not block the watcher
	go func(ctx context.Context) {
		defer func() {
			s.slotMu.Lock()
			s.watcherDownloading = false
			s.slotMu.Unlock()
		}()
		s.downloadMu.Lock()
		defer s.downloadMu.Unlock()

		// Check for context cancellation after acquiring lock
		select {
		case <-ctx.Done():
			log.Info("Download cancelled during lock acquisition")
			return
		default:
		}

		// Double-check we still need to download (another goroutine might have done it)
		s.slotMu.Lock()
		if finalizedSlot <= s.lastFinalizedSlot {
			s.slotMu.Unlock()
			return
		}
		s.slotMu.Unlock()

		startTime := time.Now()

		// Download FINALIZED state FIRST - this is what beacon relay needs
		log.WithField("slot", finalizedSlot).Debug("Downloading finalized beacon state")
		finalizedData, err := s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", finalizedSlot))
		if err != nil {
			log.WithError(err).WithField("slot", finalizedSlot).Error("Failed to download finalized beacon state")
			return
		}

		// Check for context cancellation after download
		select {
		case <-ctx.Done():
			log.Info("Download cancelled after finalized state download")
			return
		default:
		}

		// Check if a newer finalized slot appeared while downloading - if so, skip this slot
		if newerSlot := s.checkForNewerFinalizedSlot(finalizedSlot); newerSlot > 0 {
			log.WithFields(log.Fields{
				"currentSlot": finalizedSlot,
				"newerSlot":   newerSlot,
			}).Info("Newer finalized slot detected after download, skipping proof generation for old slot")
			// Update lastFinalizedSlot so we can process the newer one
			s.slotMu.Lock()
			s.lastFinalizedSlot = finalizedSlot
			s.slotMu.Unlock()
			return
		}

		// Download attested state
		log.WithField("slot", attestedSlot).Debug("Downloading attested beacon state")
		attestedData, err := s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", attestedSlot))
		if err != nil {
			log.WithError(err).WithField("slot", attestedSlot).Error("Failed to download attested beacon state")
			return
		}

		// Check for context cancellation after attested download
		select {
		case <-ctx.Done():
			log.Info("Download cancelled after attested state download")
			return
		default:
		}

		// Write to store
		err = s.store.WriteEntry(attestedSlot, finalizedSlot, attestedData, finalizedData)
		if err != nil {
			log.WithError(err).Error("Failed to write beacon states to store")
			return
		}

		// Check again before expensive proof generation
		if newerSlot := s.checkForNewerFinalizedSlot(finalizedSlot); newerSlot > 0 {
			log.WithFields(log.Fields{
				"currentSlot": finalizedSlot,
				"newerSlot":   newerSlot,
			}).Info("Newer finalized slot detected before proof generation, skipping old slot")
			s.slotMu.Lock()
			s.lastFinalizedSlot = finalizedSlot
			s.slotMu.Unlock()
			return
		}

		// Check for context cancellation before proof generation
		select {
		case <-ctx.Done():
			log.Info("Download cancelled before proof generation")
			return
		default:
		}

		// Pre-generate proofs - FINALIZED first since beacon relay needs it
		// Process ONE state at a time: generate proofs, then release memory before next
		s.preGenerateProofs(finalizedSlot, finalizedData)
		finalizedData = nil // Release finalized data before processing attested
		runtime.GC()

		// Check for context cancellation and if we should skip attested proof generation
		select {
		case <-ctx.Done():
			log.Info("Download cancelled after finalized proof generation")
			s.slotMu.Lock()
			s.lastFinalizedSlot = finalizedSlot
			s.slotMu.Unlock()
			return
		default:
		}

		if newerSlot := s.checkForNewerFinalizedSlot(finalizedSlot); newerSlot > 0 {
			log.WithFields(log.Fields{
				"currentSlot": finalizedSlot,
				"newerSlot":   newerSlot,
			}).Info("Newer finalized slot detected after finalized proofs, skipping attested proofs")
			s.slotMu.Lock()
			s.lastFinalizedSlot = finalizedSlot
			s.slotMu.Unlock()
			return
		}

		s.preGenerateProofs(attestedSlot, attestedData)
		attestedData = nil // Release attested data
		runtime.GC()

		// Update the last seen slot
		s.slotMu.Lock()
		s.lastFinalizedSlot = finalizedSlot
		s.slotMu.Unlock()

		// Prune old states
		deletedSlots, err := s.store.PruneOldStates()
		if err != nil {
			log.WithError(err).Warn("Failed to prune old states")
		} else if len(deletedSlots) > 0 {
			log.WithField("deletedSlots", deletedSlots).Debug("Pruned old beacon states")
		}

		log.WithFields(log.Fields{
			"attestedSlot":  attestedSlot,
			"finalizedSlot": finalizedSlot,
			"duration":      time.Since(startTime),
		}).Info("Successfully pre-downloaded beacon states for finalized block")
	}(ctx)

	return nil
}

// downloadCurrentFinalizedStateSync ensures beacon states and proofs are available on startup.
// First checks if states are already in the store (from previous run), and if so, just pre-generates proofs.
// Otherwise downloads states from beacon node.
// Holds downloadMu for the entire operation to ensure only ONE state is processed at a time.
func (s *Service) downloadCurrentFinalizedStateSync() error {
	update, err := s.syncer.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("get finalized update: %w", err)
	}

	attestedSlot, err := strconv.ParseUint(update.Data.AttestedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse attested slot: %w", err)
	}
	finalizedSlot, err := strconv.ParseUint(update.Data.FinalizedHeader.Beacon.Slot, 10, 64)
	if err != nil {
		return fmt.Errorf("parse finalized slot: %w", err)
	}

	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
	}).Info("Checking for beacon states on startup")

	// Acquire mutex to ensure only ONE beacon state is processed at a time
	s.downloadMu.Lock()
	defer s.downloadMu.Unlock()

	startTime := time.Now()

	// Check if states are already in store (from previous run)
	attestedData, attestedErr := s.store.GetBeaconStateData(attestedSlot)
	finalizedData, finalizedErr := s.store.GetBeaconStateData(finalizedSlot)

	if attestedErr == nil && finalizedErr == nil {
		// States already in store, just pre-generate proofs
		log.WithFields(log.Fields{
			"attestedSlot":  attestedSlot,
			"finalizedSlot": finalizedSlot,
		}).Info("Found existing states in store, pre-generating proofs")

		// Generate proofs ONE at a time - release memory between each
		s.preGenerateProofs(finalizedSlot, finalizedData)
		finalizedData = nil
		runtime.GC()

		s.preGenerateProofs(attestedSlot, attestedData)
		attestedData = nil
		runtime.GC()

		// Update the last seen slot so finality watcher doesn't re-download
		s.slotMu.Lock()
		s.lastFinalizedSlot = finalizedSlot
		s.slotMu.Unlock()

		log.WithFields(log.Fields{
			"attestedSlot":  attestedSlot,
			"finalizedSlot": finalizedSlot,
			"duration":      time.Since(startTime),
		}).Info("Proofs generated from existing states")

		return nil
	}

	// States not in store, download from beacon node
	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
	}).Info("Downloading initial beacon states")

	// Download FINALIZED state FIRST - this is what beacon relay needs
	log.WithField("slot", finalizedSlot).Info("Downloading finalized beacon state")
	finalizedData, err = s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", finalizedSlot))
	if err != nil {
		return fmt.Errorf("download finalized state at slot %d: %w", finalizedSlot, err)
	}

	// Download attested state
	log.WithField("slot", attestedSlot).Info("Downloading attested beacon state")
	attestedData, err = s.syncer.Client.GetBeaconState(fmt.Sprintf("%d", attestedSlot))
	if err != nil {
		return fmt.Errorf("download attested state at slot %d: %w", attestedSlot, err)
	}

	// Write to store (INSERT OR IGNORE handles duplicates)
	err = s.store.WriteEntry(attestedSlot, finalizedSlot, attestedData, finalizedData)
	if err != nil {
		return fmt.Errorf("write states to store: %w", err)
	}

	// Update the last seen slot so finality watcher doesn't re-download
	s.slotMu.Lock()
	s.lastFinalizedSlot = finalizedSlot
	s.slotMu.Unlock()

	// Pre-generate proofs ONE at a time - release memory between each
	s.preGenerateProofs(finalizedSlot, finalizedData)
	finalizedData = nil
	runtime.GC()

	s.preGenerateProofs(attestedSlot, attestedData)
	attestedData = nil
	runtime.GC()

	log.WithFields(log.Fields{
		"attestedSlot":  attestedSlot,
		"finalizedSlot": finalizedSlot,
		"duration":      time.Since(startTime),
	}).Info("Initial beacon states downloaded and cached")

	return nil
}

// preGenerateProofs generates and caches all proofs for a slot from state data.
// This is called by the finality watcher after downloading states, so proofs are
// ready before the beacon relay needs them.
// Note: This function is called while downloadMu is held by the finality watcher,
// so it's already serialized with other downloads/proof generations.
//
// Memory optimization: Uses the lite SSZ unmarshaler which saves ~130MB+ by:
// - Only extracting fields needed for proof generation (BlockRoots, Checkpoints, SyncCommittees)
// - Computing hashes for large fields (Validators, Balances, Participation) without storing them
func (s *Service) preGenerateProofs(slot uint64, data []byte) {
	// Check if proofs are already cached
	if s.hasAllProofsCached(slot) {
		log.WithField("slot", slot).Debug("Proofs already cached, skipping pre-generation")
		return
	}

	startTime := time.Now()
	log.WithField("slot", slot).Info("Pre-generating proofs for slot (using lite unmarshaler)")

	// Use lite unmarshaler to save ~130MB+ of memory per state
	unmarshalStart := time.Now()
	beaconState, err := s.unmarshalBeaconStateLite(slot, data)
	// Release raw data reference to help GC
	data = nil
	if err != nil {
		log.WithError(err).WithField("slot", slot).Warn("Failed to unmarshal beacon state for proof pre-generation")
		return
	}
	unmarshalDuration := time.Since(unmarshalStart)

	treeStart := time.Now()
	tree, err := beaconState.GetTree()
	if err != nil {
		log.WithError(err).WithField("slot", slot).Warn("Failed to get state tree for proof pre-generation")
		return
	}
	treeDuration := time.Since(treeStart)

	// Note: tree.Hash() was previously called here but is redundant.
	// tree.Prove() internally computes sibling hashes on-demand via hashNode(),
	// and hashNode() does not cache - it recomputes on every call. So calling
	// tree.Hash() first just adds extra work. Better to let Prove() compute
	// only the hashes it needs along each proof path.

	proofsStart := time.Now()
	s.cacheAllProofs(slot, beaconState, tree)
	proofsDuration := time.Since(proofsStart)

	// Release large objects and force GC to prevent memory buildup
	beaconState = nil
	tree = nil
	runtime.GC()

	log.WithFields(log.Fields{
		"slot":             slot,
		"unmarshalMs":      unmarshalDuration.Milliseconds(),
		"treeMs":           treeDuration.Milliseconds(),
		"proofsMs":         proofsDuration.Milliseconds(),
		"totalMs":          time.Since(startTime).Milliseconds(),
	}).Info("Pre-generated and cached proofs for slot")
}
