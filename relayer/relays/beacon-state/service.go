package beaconstate

import (
	"context"
	"fmt"
	"net/http"
	"time"

	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
)

type Service struct {
	config       *Config
	protocol     *protocol.Protocol
	beaconClient *api.BeaconClient
	stateCache   *StateCache
	proofCache   *ProofCache
	httpServer   *http.Server
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

	// Initialize beacon API client
	s.beaconClient = api.NewBeaconClient(s.config.Beacon.Endpoint, s.config.Beacon.StateEndpoint)

	// Initialize caches
	stateTTL := time.Duration(s.config.Cache.StateTTLSeconds) * time.Second
	proofTTL := time.Duration(s.config.Cache.ProofTTLSeconds) * time.Second
	s.stateCache = NewStateCache(s.config.Cache.MaxStates, stateTTL)
	s.proofCache = NewProofCache(s.config.Cache.MaxProofs, proofTTL)

	// Parse timeouts
	readTimeout, _ := time.ParseDuration(s.config.HTTP.ReadTimeout)
	writeTimeout, _ := time.ParseDuration(s.config.HTTP.WriteTimeout)

	// Setup HTTP server
	mux := http.NewServeMux()
	mux.HandleFunc("/health", s.handleHealth)
	mux.HandleFunc("/v1/proofs/finalized-header", s.handleFinalizedHeaderProof)
	mux.HandleFunc("/v1/proofs/execution-state-root", s.handleExecutionStateRootProof)
	mux.HandleFunc("/v1/proofs/block-root", s.handleBlockRootProof)
	mux.HandleFunc("/v1/proofs/sync-committee", s.handleSyncCommitteeProof)

	s.httpServer = &http.Server{
		Addr:         fmt.Sprintf(":%d", s.config.HTTP.Port),
		Handler:      mux,
		ReadTimeout:  readTimeout,
		WriteTimeout: writeTimeout,
	}

	// Start HTTP server in errgroup
	eg.Go(func() error {
		log.WithField("port", s.config.HTTP.Port).Info("Starting beacon state service HTTP server")
		err := s.httpServer.ListenAndServe()
		if err != nil && err != http.ErrServerClosed {
			return fmt.Errorf("http server: %w", err)
		}
		return nil
	})

	// Graceful shutdown
	eg.Go(func() error {
		<-ctx.Done()
		log.Info("Shutting down beacon state service HTTP server")
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return s.httpServer.Shutdown(shutdownCtx)
	})

	return nil
}

func (s *Service) GetProtocol() *protocol.Protocol {
	return s.protocol
}

func (s *Service) GetStateCache() *StateCache {
	return s.stateCache
}

func (s *Service) GetProofCache() *ProofCache {
	return s.proofCache
}
