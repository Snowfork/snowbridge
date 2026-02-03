package beaconstate

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	beaconerrors "github.com/snowfork/snowbridge/relayer/relays/beacon/errors"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

// ErrProofNotReady is an alias for the shared error - kept for backwards compatibility
var ErrProofNotReady = beaconerrors.ErrProofNotReady

type Client struct {
	endpoint   string
	httpClient *http.Client
}

func NewClient(endpoint string) *Client {
	return &Client{
		endpoint: endpoint,
		httpClient: &http.Client{
			Timeout: 60 * time.Second,
		},
	}
}

// GetFinalizedHeaderProof fetches the finalized header proof for a slot
// Returns ErrProofNotReady if the proof is not yet cached (503 response).
func (c *Client) GetFinalizedHeaderProof(slot uint64) ([]types.H256, error) {
	url := fmt.Sprintf("%s/v1/proofs/finalized-header?slot=%d", c.endpoint, slot)
	proofResp, err := c.fetchProof(url)
	if err != nil {
		return nil, err
	}

	// Parse proof to []types.H256
	proof := make([]types.H256, len(proofResp.Proof))
	for i, p := range proofResp.Proof {
		proof[i], err = util.HexToH256(p)
		if err != nil {
			return nil, fmt.Errorf("parse proof[%d]: %w", i, err)
		}
	}

	return proof, nil
}

// GetBlockRootProof fetches the block root proof for a slot and returns a scale.BlockRootProof
// that includes the block roots tree for ancestry proofs.
// Returns ErrProofNotReady if the proof is not yet cached (503 response).
func (c *Client) GetBlockRootProof(slot uint64) (*scale.BlockRootProof, error) {
	url := fmt.Sprintf("%s/v1/proofs/block-root?slot=%d", c.endpoint, slot)
	log.WithField("url", url).Debug("Fetching block root proof from beacon state service")

	resp, err := c.httpClient.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	if resp.StatusCode == http.StatusServiceUnavailable {
		return nil, ErrProofNotReady
	}

	if resp.StatusCode != http.StatusOK {
		var errResp ErrorResponse
		if json.Unmarshal(body, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("beacon state service error: %s", errResp.Error)
		}
		return nil, fmt.Errorf("beacon state service returned status %d", resp.StatusCode)
	}

	var blockRootResp BlockRootProofResponse
	if err := json.Unmarshal(body, &blockRootResp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	// Parse leaf
	leaf, err := util.HexToH256(blockRootResp.Leaf)
	if err != nil {
		return nil, fmt.Errorf("parse leaf: %w", err)
	}

	// Parse proof
	proof := make([]types.H256, len(blockRootResp.Proof))
	for i, p := range blockRootResp.Proof {
		proof[i], err = util.HexToH256(p)
		if err != nil {
			return nil, fmt.Errorf("parse proof[%d]: %w", i, err)
		}
	}

	// Parse block roots and build tree
	blockRoots := make([][]byte, len(blockRootResp.BlockRoots))
	for i, root := range blockRootResp.BlockRoots {
		h, err := util.HexStringTo32Bytes(root)
		if err != nil {
			return nil, fmt.Errorf("parse block root[%d]: %w", i, err)
		}
		blockRoots[i] = h[:]
	}

	// Build block roots tree for ancestry proofs
	blockRootsContainer := &state.BlockRootsContainerMainnet{}
	blockRootsContainer.SetBlockRoots(blockRoots)
	tree, err := blockRootsContainer.GetTree()
	if err != nil {
		return nil, fmt.Errorf("build block roots tree: %w", err)
	}

	return &scale.BlockRootProof{
		Leaf:  leaf,
		Proof: proof,
		Tree:  tree,
	}, nil
}

// GetSyncCommitteeProof fetches the sync committee proof for a slot including pubkeys
// Returns ErrProofNotReady if the proof is not yet cached (503 response).
func (c *Client) GetSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error) {
	url := fmt.Sprintf("%s/v1/proofs/sync-committee?slot=%d&period=%s", c.endpoint, slot, period)
	log.WithField("url", url).Debug("Fetching sync committee proof from beacon state service")

	resp, err := c.httpClient.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	if resp.StatusCode == http.StatusServiceUnavailable {
		return nil, ErrProofNotReady
	}

	if resp.StatusCode != http.StatusOK {
		var errResp ErrorResponse
		if json.Unmarshal(body, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("beacon state service error: %s", errResp.Error)
		}
		return nil, fmt.Errorf("beacon state service returned status %d", resp.StatusCode)
	}

	var proofResp SyncCommitteeProofResponse
	if err := json.Unmarshal(body, &proofResp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	// Parse proof
	proof := make([]types.H256, len(proofResp.Proof))
	for i, p := range proofResp.Proof {
		proof[i], err = util.HexToH256(p)
		if err != nil {
			return nil, fmt.Errorf("parse proof[%d]: %w", i, err)
		}
	}

	// Parse pubkeys
	pubkeys := make([][48]byte, len(proofResp.Pubkeys))
	for i, pk := range proofResp.Pubkeys {
		pkBytes, err := util.HexStringToByteArray(pk)
		if err != nil {
			return nil, fmt.Errorf("parse pubkey[%d]: %w", i, err)
		}
		if len(pkBytes) != 48 {
			return nil, fmt.Errorf("invalid pubkey length at index %d: got %d, want 48", i, len(pkBytes))
		}
		copy(pubkeys[i][:], pkBytes)
	}

	// Parse aggregate pubkey
	aggPkBytes, err := util.HexStringToByteArray(proofResp.AggregatePubkey)
	if err != nil {
		return nil, fmt.Errorf("parse aggregate pubkey: %w", err)
	}
	var aggPk [48]byte
	if len(aggPkBytes) != 48 {
		return nil, fmt.Errorf("invalid aggregate pubkey length: got %d, want 48", len(aggPkBytes))
	}
	copy(aggPk[:], aggPkBytes)

	return &scale.SyncCommitteeProof{
		Pubkeys:         pubkeys,
		AggregatePubkey: aggPk,
		Proof:           proof,
	}, nil
}

// Health checks if the beacon state service is healthy
func (c *Client) Health() error {
	url := fmt.Sprintf("%s/health", c.endpoint)
	resp, err := c.httpClient.Get(url)
	if err != nil {
		return fmt.Errorf("health check failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("health check returned status %d", resp.StatusCode)
	}
	return nil
}

func (c *Client) fetchProof(url string) (*ProofResponse, error) {
	log.WithField("url", url).Debug("Fetching proof from beacon state service")

	resp, err := c.httpClient.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	if resp.StatusCode == http.StatusServiceUnavailable {
		return nil, ErrProofNotReady
	}

	if resp.StatusCode != http.StatusOK {
		var errResp ErrorResponse
		if json.Unmarshal(body, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("beacon state service error: %s", errResp.Error)
		}
		return nil, fmt.Errorf("beacon state service returned status %d", resp.StatusCode)
	}

	var proofResp ProofResponse
	if err := json.Unmarshal(body, &proofResp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	return &proofResp, nil
}

