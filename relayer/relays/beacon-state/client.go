package beaconstate

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

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
func (c *Client) GetFinalizedHeaderProof(slot uint64) (*ProofResponse, error) {
	url := fmt.Sprintf("%s/v1/proofs/finalized-header?slot=%d", c.endpoint, slot)
	return c.fetchProof(url)
}

// GetExecutionStateRootProof fetches the execution state root proof for a slot
func (c *Client) GetExecutionStateRootProof(slot uint64) (*ProofResponse, error) {
	url := fmt.Sprintf("%s/v1/proofs/execution-state-root?slot=%d", c.endpoint, slot)
	return c.fetchProof(url)
}

// GetBlockRootProof fetches the block root proof for a slot and returns a scale.BlockRootProof
// that includes the block roots tree for ancestry proofs
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

// GetBlockRootProofRaw fetches the raw block root proof response
func (c *Client) GetBlockRootProofRaw(slot uint64) (*BlockRootProofResponse, error) {
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

	return &blockRootResp, nil
}

// GetSyncCommitteeProof fetches the sync committee proof for a slot
func (c *Client) GetSyncCommitteeProof(slot uint64, period string) (*ProofResponse, error) {
	url := fmt.Sprintf("%s/v1/proofs/sync-committee?slot=%d&period=%s", c.endpoint, slot, period)
	return c.fetchProof(url)
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

// GetBeaconState fetches raw SSZ beacon state data for a slot
func (c *Client) GetBeaconState(slot uint64) ([]byte, error) {
	url := fmt.Sprintf("%s/v1/state?slot=%d", c.endpoint, slot)
	log.WithField("url", url).Debug("Fetching beacon state from beacon state service")

	resp, err := c.httpClient.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		var errResp ErrorResponse
		if json.Unmarshal(body, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("beacon state service error: %s", errResp.Error)
		}
		return nil, fmt.Errorf("beacon state service returned status %d", resp.StatusCode)
	}

	var stateResp StateResponse
	if err := json.Unmarshal(body, &stateResp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	// Decode hex data
	data, err := util.HexStringToByteArray(stateResp.Data)
	if err != nil {
		return nil, fmt.Errorf("decode state data: %w", err)
	}

	return data, nil
}

// GetBeaconStateInRange fetches beacon states within a slot range from the persistent store
func (c *Client) GetBeaconStateInRange(minSlot, maxSlot uint64) (*StateRangeResponse, error) {
	url := fmt.Sprintf("%s/v1/state/range?minSlot=%d&maxSlot=%d", c.endpoint, minSlot, maxSlot)
	log.WithField("url", url).Debug("Fetching beacon state range from beacon state service")

	resp, err := c.httpClient.Get(url)
	if err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		var errResp ErrorResponse
		if json.Unmarshal(body, &errResp) == nil && errResp.Error != "" {
			return nil, fmt.Errorf("beacon state service error: %s", errResp.Error)
		}
		return nil, fmt.Errorf("beacon state service returned status %d", resp.StatusCode)
	}

	var rangeResp StateRangeResponse
	if err := json.Unmarshal(body, &rangeResp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	return &rangeResp, nil
}

// GetBeaconStateInRangeDecoded fetches beacon states within a slot range and decodes the hex data
func (c *Client) GetBeaconStateInRangeDecoded(minSlot, maxSlot uint64) (attestedSlot, finalizedSlot uint64, attestedData, finalizedData []byte, err error) {
	rangeResp, err := c.GetBeaconStateInRange(minSlot, maxSlot)
	if err != nil {
		return 0, 0, nil, nil, err
	}

	attestedData, err = util.HexStringToByteArray(rangeResp.AttestedData)
	if err != nil {
		return 0, 0, nil, nil, fmt.Errorf("decode attested data: %w", err)
	}

	finalizedData, err = util.HexStringToByteArray(rangeResp.FinalizedData)
	if err != nil {
		return 0, 0, nil, nil, fmt.Errorf("decode finalized data: %w", err)
	}

	return rangeResp.AttestedSlot, rangeResp.FinalizedSlot, attestedData, finalizedData, nil
}
