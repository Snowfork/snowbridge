package syncer

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
)

const (
	ConstructRequestErrorMessage = "construct header request"
	DoHTTPRequestErrorMessage    = "do http request"
	HTTPStatusNotOKErrorMessage  = "http status not ok"
	ReadResponseBodyErrorMessage = "read response body"
	UnmarshalBodyErrorMessage    = "unmarshal body"
)

type BeaconClientTracker interface {
	GetHeader(blockRoot common.Hash) (BeaconHeader, error)
	GetHeaderBySlot(slot uint64) (BeaconHeader, error)
	GetSyncCommitteePeriodUpdate(from uint64) (SyncCommitteePeriodUpdateResponse, error)
	GetBeaconBlock(slot uint64) (BeaconBlockResponse, error)
	GetInitialSync(blockRoot string) (BootstrapResponse, error)
	GetFinalizedCheckpoint() (FinalizedCheckpoint, error)
	GetGenesis() (Genesis, error)
	GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error)
	GetBootstrap(blockRoot common.Hash) (Bootstrap, error)
}

var (
	ErrNotFound                        = errors.New("not found")
	ErrSyncCommitteeUpdateNotAvailable = errors.New("no sync committee update available")
)

type BeaconClient struct {
	httpClient http.Client
	endpoint   string
}

func NewBeaconClient(endpoint string) *BeaconClient {
	return &BeaconClient{
		http.Client{},
		endpoint,
	}
}

type BeaconHeader struct {
	Slot          uint64      `json:"slot"`
	ProposerIndex uint64      `json:"proposer_index"`
	ParentRoot    common.Hash `json:"parent_root"`
	StateRoot     common.Hash `json:"state_root"`
	BodyRoot      common.Hash `json:"body_root"`
}

type Bootstrap struct {
	Header                     BeaconHeaderJSON
	CurrentSyncCommittee       SyncCommitteeJSON
	CurrentSyncCommitteeBranch []string
}

type Genesis struct {
	ValidatorsRoot common.Hash
	Time           uint64
}

type BeaconBlock struct {
	Slot          uint64
	ProposerIndex uint64
	ParentRoot    common.Hash
	StateRoot     common.Hash
	BodyRoot      common.Hash
}

type FinalizedCheckpoint struct {
	FinalizedBlockRoot common.Hash
}

func (h *HeaderResponse) ToBeaconHeader() (BeaconHeader, error) {
	slot, err := toUint64(h.Slot)
	if err != nil {
		return BeaconHeader{}, err
	}

	proposerIndex, err := toUint64(h.ProposerIndex)
	if err != nil {
		return BeaconHeader{}, err
	}

	return BeaconHeader{
		Slot:          slot,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(h.ParentRoot),
		StateRoot:     common.HexToHash(h.StateRoot),
		BodyRoot:      common.HexToHash(h.BodyRoot),
	}, nil
}

func (b *BeaconClient) GetBootstrap(blockRoot common.Hash) (Bootstrap, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/bootstrap/%s", b.endpoint, blockRoot), nil)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return Bootstrap{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response BootstrapResponse
	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	slot, err := toUint64(response.Data.Header.Beacon.Slot)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("convert slot to int: %w", err)
	}

	proposerIndex, err := toUint64(response.Data.Header.Beacon.ProposerIndex)
	if err != nil {
		return Bootstrap{}, fmt.Errorf("convert proposer index to int: %w", err)
	}

	beaconHeader := BeaconHeaderJSON{
		Slot:          slot,
		ProposerIndex: proposerIndex,
		ParentRoot:    response.Data.Header.Beacon.ParentRoot,
		StateRoot:     response.Data.Header.Beacon.StateRoot,
		BodyRoot:      response.Data.Header.Beacon.BodyRoot,
	}

	return Bootstrap{
		Header:                     beaconHeader,
		CurrentSyncCommittee:       SyncCommitteeJSON(response.Data.CurrentSyncCommittee),
		CurrentSyncCommitteeBranch: response.Data.CurrentSyncCommitteeBranch,
	}, nil
}

func (b *BeaconClient) GetGenesis() (Genesis, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/genesis", b.endpoint), nil)
	if err != nil {
		return Genesis{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return Genesis{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return Genesis{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return Genesis{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response GenesisResponse
	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return Genesis{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	time, err := toUint64(response.Data.Time)
	if err != nil {
		return Genesis{}, fmt.Errorf("convert genesis time string to uint64: %w", err)
	}

	return Genesis{
		ValidatorsRoot: common.HexToHash(response.Data.GenesisValidatorsRoot),
		Time:           time,
	}, nil
}

func (b *BeaconClient) GetFinalizedCheckpoint() (FinalizedCheckpoint, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/head/finality_checkpoints", b.endpoint), nil)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %d", ReadResponseBodyErrorMessage, res.StatusCode)
	}

	var response FinalizedCheckpointResponse
	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return FinalizedCheckpoint{
		FinalizedBlockRoot: common.HexToHash(response.Data.Finalized.Root),
	}, nil
}

func (b *BeaconClient) GetHeaderBySlot(slot uint64) (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/headers/%d", b.endpoint, slot), nil)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return BeaconHeader{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response BeaconHeaderResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	slotFromResponse, err := strconv.ParseUint(response.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := strconv.ParseUint(response.Data.Header.Message.ProposerIndex, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse proposerIndex as int: %w", err)
	}

	return BeaconHeader{
		Slot:          slotFromResponse,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(response.Data.Header.Message.ParentRoot),
		StateRoot:     common.HexToHash(response.Data.Header.Message.StateRoot),
		BodyRoot:      common.HexToHash(response.Data.Header.Message.BodyRoot),
	}, nil
}

func (b *BeaconClient) GetHeader(blockRoot common.Hash) (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/headers/%s", b.endpoint, blockRoot), nil)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return BeaconHeader{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response BeaconHeaderResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	slotScale, err := strconv.ParseUint(response.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := strconv.ParseUint(response.Data.Header.Message.ProposerIndex, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse proposerIndex as int: %w", err)
	}

	return BeaconHeader{
		Slot:          slotScale,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(response.Data.Header.Message.ParentRoot),
		StateRoot:     common.HexToHash(response.Data.Header.Message.StateRoot),
		BodyRoot:      common.HexToHash(response.Data.Header.Message.BodyRoot),
	}, nil
}

func (b *BeaconClient) GetBeaconBlock(blockID common.Hash) (BeaconBlockResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.endpoint, blockID), nil)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response, nil
}

func (b *BeaconClient) GetBeaconBlockBySlot(slot uint64) (BeaconBlockResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", b.endpoint, slot), nil)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return BeaconBlockResponse{}, ErrNotFound
		}

		return BeaconBlockResponse{}, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response, nil
}

func (b *BeaconClient) GetBeaconBlockRoot(slot uint64) (common.Hash, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/blocks/%d/root", b.endpoint, slot), nil)
	if err != nil {
		return common.Hash{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return common.Hash{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return common.Hash{}, ErrNotFound
		}

		return common.Hash{}, fmt.Errorf("fetch beacon block %d: %s", res.StatusCode, HTTPStatusNotOKErrorMessage)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return common.Hash{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response struct {
		Data struct {
			Root string `json:"root"`
		} `json:"data"`
	}

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return common.Hash{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return common.HexToHash(response.Data.Root), nil
}

func (b *BeaconClient) GetSyncCommitteePeriodUpdate(from uint64) (SyncCommitteePeriodUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/updates?start_period=%d&count=1", b.endpoint, from), nil)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, err := io.ReadAll(res.Body)
		if err != nil {
			return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", HTTPStatusNotOKErrorMessage, err)
		}

		var response ErrorMessage

		err = json.Unmarshal(bodyBytes, &response)
		if err != nil {
			return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", HTTPStatusNotOKErrorMessage, err)
		}

		if strings.Contains(response.Message, "No partialUpdate available") {
			return SyncCommitteePeriodUpdateResponse{}, ErrSyncCommitteeUpdateNotAvailable
		}

		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s :%d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response []SyncCommitteePeriodUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	if len(response) == 0 {
		return SyncCommitteePeriodUpdateResponse{}, ErrNotFound
	}

	return response[0], nil
}

type ForkResponse struct {
	Data struct {
		PreviousVersion string `json:"previous_version"`
		CurrentVersion  string `json:"current_version"`
		Epoch           string `json:"epoch"`
	} `json:"data"`
}

type LatestFinalisedUpdateResponse struct {
	Data struct {
		AttestedHeader struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"attested_header"`
		FinalizedHeader struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"finalized_header"`
		FinalityBranch []common.Hash         `json:"finality_branch"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
		SignatureSlot  string                `json:"signature_slot"`
	} `json:"data"`
}

func (b *BeaconClient) GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/finality_update", b.endpoint), nil)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %d", DoHTTPRequestErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response LatestFinalisedUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response, nil
}

func (b *BeaconClient) DownloadBeaconState(stateIdOrSlot string) error {
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/eth/v2/debug/beacon/states/%s", b.endpoint, stateIdOrSlot), nil)
	if err != nil {
		return err
	}
	req.Header.Add("Accept", "application/octet-stream")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return err
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return ErrNotFound
		}

		return fmt.Errorf("%s: %d", DoHTTPRequestErrorMessage, res.StatusCode)
	}

	defer res.Body.Close()
	out, err := os.Create("beacon_state.ssz")
	if err != nil {
		return err
	}

	defer out.Close()
	io.Copy(out, res.Body)

	return nil
}

type LatestHeaderUpdateResponse struct {
	Data struct {
		AttestedHeader HeaderResponse        `json:"attested_header"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
	} `json:"data"`
}
