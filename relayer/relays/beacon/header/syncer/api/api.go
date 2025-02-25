package api

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

var (
	ErrNotFound                        = errors.New("not found")
	ErrSyncCommitteeUpdateNotAvailable = errors.New("no sync committee update available")
	ConstructRequestErrorMessage       = "construct header request"
	DoHTTPRequestErrorMessage          = "do http request"
	HTTPStatusNotOKErrorMessage        = "http status not ok"
	ReadResponseBodyErrorMessage       = "read response body"
	UnmarshalBodyErrorMessage          = "unmarshal body"
)

type BeaconAPI interface {
	GetBootstrap(blockRoot common.Hash) (BootstrapResponse, error)
	GetGenesis() (Genesis, error)
	GetFinalizedCheckpoint() (FinalizedCheckpoint, error)
	GetHeaderBySlot(slot uint64) (BeaconHeader, error)
	GetHeaderAtHead() (BeaconHeader, error)
	GetHeaderByBlockRoot(blockRoot common.Hash) (BeaconHeader, error)
	GetBeaconBlockBySlot(slot uint64) (BeaconBlockResponse, error)
	GetBeaconBlockRoot(slot uint64) (common.Hash, error)
	GetBeaconBlock(blockID common.Hash) (BeaconBlockResponse, error)
	GetBeaconBlockBytes(blockID common.Hash) ([]byte, error)
	GetSyncCommitteePeriodUpdate(from uint64) (SyncCommitteePeriodUpdateResponse, error)
	GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error)
	GetBeaconState(stateIdOrSlot string) ([]byte, error)
}

type BeaconClient struct {
	httpClient    http.Client
	endpoint      string
	stateEndpoint string
}

func NewBeaconClient(endpoint, stateEndpoint string) *BeaconClient {
	return &BeaconClient{
		http.Client{},
		endpoint,
		stateEndpoint,
	}
}

func (b *BeaconClient) GetBootstrap(blockRoot common.Hash) (BootstrapResponse, error) {
	var response BootstrapResponse
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/bootstrap/%s", b.endpoint, blockRoot), nil)
	if err != nil {
		return response, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return response, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return response, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return response, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return response, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response, nil
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

	time, err := util.ToUint64(response.Data.Time)
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
	return b.GetHeader(fmt.Sprintf("%d", slot))
}

func (b *BeaconClient) GetHeaderAtHead() (BeaconHeader, error) {
	return b.GetHeader("head")
}

func (b *BeaconClient) GetHeaderByBlockRoot(blockRoot common.Hash) (BeaconHeader, error) {
	return b.GetHeader(blockRoot.Hex())
}

func (b *BeaconClient) GetHeader(qualifier string) (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/headers/%s", b.endpoint, qualifier), nil)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return BeaconHeader{}, ErrNotFound
		}

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

func (b *BeaconClient) GetBeaconBlock(blockID common.Hash) (BeaconBlockResponse, error) {
	var beaconBlockResponse BeaconBlockResponse

	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.endpoint, blockID), nil)
	if err != nil {
		return beaconBlockResponse, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Add("Accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return beaconBlockResponse, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return beaconBlockResponse, fmt.Errorf("%s: %d", HTTPStatusNotOKErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return beaconBlockResponse, fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	err = json.Unmarshal(bodyBytes, &beaconBlockResponse)
	if err != nil {
		return beaconBlockResponse, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}
	return beaconBlockResponse, nil
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

func (b *BeaconClient) GetBeaconState(stateIdOrSlot string) ([]byte, error) {
	var data []byte
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/eth/v2/debug/beacon/states/%s", b.stateEndpoint, stateIdOrSlot), nil)
	if err != nil {
		return data, err
	}

	req.Header.Add("Accept", "application/octet-stream")

	startTime := time.Now()
	res, err := b.httpClient.Do(req)
	endTime := time.Now()
	duration := endTime.Sub(startTime)
	log.WithFields(log.Fields{"startTime": startTime.Format(time.UnixDate), "endTime": endTime.Format(time.UnixDate), "duration": duration.Seconds()}).Info("beacon state download time")

	if err != nil {
		return data, err
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return data, ErrNotFound
		}

		return data, fmt.Errorf("%s: %d", DoHTTPRequestErrorMessage, res.StatusCode)
	}

	buf := new(bytes.Buffer)
	buf.ReadFrom(res.Body)
	data = buf.Bytes()
	return data, nil
}

func (b *BeaconClient) GetBeaconBlockBytes(blockID common.Hash) ([]byte, error) {
	var data []byte
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.stateEndpoint, blockID), nil)
	if err != nil {
		return data, err
	}

	req.Header.Add("Accept", "application/octet-stream")

	res, err := b.httpClient.Do(req)
	if err != nil {
		return data, err
	}

	if res.StatusCode != http.StatusOK {
		if res.StatusCode == 404 {
			return data, ErrNotFound
		}

		return data, fmt.Errorf("%s: %d", DoHTTPRequestErrorMessage, res.StatusCode)
	}

	buf := new(bytes.Buffer)
	buf.ReadFrom(res.Body)
	data = buf.Bytes()
	return data, nil
}
