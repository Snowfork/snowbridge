package api

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"strconv"
	"strings"
	"sync"
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

type FinalizedCheckpointEvent struct {
	Block string `json:"block"`
	State string `json:"state"`
	Epoch string `json:"epoch"`
}

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
	SubscribeFinalizedCheckpoint(ctx context.Context) (<-chan FinalizedCheckpointEvent, <-chan error, error)
}

type BeaconClient struct {
	httpClient http.Client
	endpoints  []string
	currentIdx int
	mu         sync.Mutex
}

func NewBeaconClient(endpoints []string) *BeaconClient {
	return &BeaconClient{
		httpClient: http.Client{},
		endpoints:  endpoints,
		currentIdx: rand.Intn(len(endpoints)),
	}
}

func (b *BeaconClient) endpoint() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.endpoints[b.currentIdx]
}

func (b *BeaconClient) do(req *http.Request) (*http.Response, error) {
	res, err := b.httpClient.Do(req)
	if err != nil {
		b.switchEndpoint()
		return nil, err
	}
	if res.StatusCode >= 500 {
		b.switchEndpoint()
	}
	return res, nil
}

func (b *BeaconClient) switchEndpoint() {
	b.mu.Lock()
	defer b.mu.Unlock()
	if len(b.endpoints) <= 1 {
		return
	}
	prev := b.endpoints[b.currentIdx]
	b.currentIdx = (b.currentIdx + 1) % len(b.endpoints)
	log.WithField("endpoint", prev).Warn("Beacon endpoint failed")
	log.WithField("endpoint", b.endpoints[b.currentIdx]).Info("Switched beacon endpoint")
}

func (b *BeaconClient) GetBootstrap(blockRoot common.Hash) (BootstrapResponse, error) {
	var response BootstrapResponse
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/bootstrap/%s", b.endpoint(), blockRoot), nil)
	if err != nil {
		return response, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/genesis", b.endpoint()), nil)
	if err != nil {
		return Genesis{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/head/finality_checkpoints", b.endpoint()), nil)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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

	epoch, err := strconv.ParseUint(response.Data.Finalized.Epoch, 10, 64)
	if err != nil {
		return FinalizedCheckpoint{}, fmt.Errorf("parse finalized checkpoint epoch: %w", err)
	}

	return FinalizedCheckpoint{
		FinalizedBlockRoot: common.HexToHash(response.Data.Finalized.Root),
		Epoch:              epoch,
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/headers/%s", b.endpoint(), qualifier), nil)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", b.endpoint(), slot), nil)
	if err != nil {
		return BeaconBlockResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/blocks/%d/root", b.endpoint(), slot), nil)
	if err != nil {
		return common.Hash{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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

	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.endpoint(), blockID), nil)
	if err != nil {
		return beaconBlockResponse, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Add("Accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/updates?start_period=%d&count=1", b.endpoint(), from), nil)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/finality_update", b.endpoint()), nil)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.do(req)
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	// A node that does not serve the light client API for the current fork answers 404. The
	// slots it would have reported are also available from the standard header endpoints, and
	// every proof the relayer submits is built from beacon state rather than from this
	// response, so fall back rather than stall.
	if res.StatusCode == http.StatusNotFound {
		return b.finalizedUpdateFromHeaders()
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

// finalizedUpdateFromHeaders reports the finalized and attested slots from the standard
// header endpoints.
//
// The result carries no finality branch and no sync aggregate, because neither is available
// this way. It is marked DerivedFromHeaders so the one path that needs them can refuse it
// instead of building an update out of zero values.
func (b *BeaconClient) finalizedUpdateFromHeaders() (LatestFinalisedUpdateResponse, error) {
	finalized, err := b.GetHeader("finalized")
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("get finalized header: %w", err)
	}
	head, err := b.GetHeaderAtHead()
	if err != nil {
		return LatestFinalisedUpdateResponse{}, fmt.Errorf("get head header: %w", err)
	}

	var response LatestFinalisedUpdateResponse
	response.DerivedFromHeaders = true
	response.Data.FinalizedHeader.Beacon = finalized.ToResponse()
	response.Data.AttestedHeader.Beacon = head.ToResponse()
	response.Data.SignatureSlot = strconv.FormatUint(head.Slot+1, 10)
	return response, nil
}

func (b *BeaconClient) GetBeaconState(stateIdOrSlot string) ([]byte, error) {
	var data []byte
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/eth/v2/debug/beacon/states/%s", b.endpoint(), stateIdOrSlot), nil)
	if err != nil {
		return data, err
	}

	req.Header.Add("Accept", "application/octet-stream")

	startTime := time.Now()
	res, err := b.do(req)
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
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/eth/v2/beacon/blocks/%s", b.endpoint(), blockID), nil)
	if err != nil {
		return data, err
	}

	req.Header.Add("Accept", "application/octet-stream")

	res, err := b.do(req)
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

func (b *BeaconClient) SubscribeFinalizedCheckpoint(ctx context.Context) (<-chan FinalizedCheckpointEvent, <-chan error, error) {
	endpoint := b.endpoint()
	url := fmt.Sprintf("%s/eth/v1/events?topics=finalized_checkpoint", endpoint)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, nil, fmt.Errorf("create request for finalized checkpoint stream: %w", err)
	}
	req.Header.Set("Accept", "text/event-stream")

	// Call the endpoint directly without b.do() because we want to stream the response
	res, err := b.httpClient.Do(req)
	if err != nil {
		b.switchEndpoint()
		return nil, nil, fmt.Errorf("dial finalized checkpoint stream: %w", err)
	}

	if res.StatusCode != http.StatusOK {
		res.Body.Close()
		b.switchEndpoint()
		return nil, nil, fmt.Errorf("finalized checkpoint stream returned status: %d", res.StatusCode)
	}

	eventsCh := make(chan FinalizedCheckpointEvent, 10)
	errCh := make(chan error, 1)

	go func() {
		defer res.Body.Close()
		reader := bufio.NewReader(res.Body)

		var eventType string
		for {
			line, err := reader.ReadString('\n')
			if err != nil {
				select {
				case <-ctx.Done():
				case errCh <- err:
				default:
				}
				return
			}

			line = strings.TrimSpace(line)
			if line == "" {
				// Empty line means event block complete
				eventType = ""
				continue
			}

			if strings.HasPrefix(line, ":") {
				// SSE comment, ignore
				continue
			}

			parts := strings.SplitN(line, ":", 2)
			key := parts[0]
			var value string
			if len(parts) > 1 {
				value = strings.TrimSpace(parts[1])
			}

			switch key {
			case "event":
				eventType = value
			case "data":
				if eventType == "finalized_checkpoint" {
					var event FinalizedCheckpointEvent
					if err := json.Unmarshal([]byte(value), &event); err != nil {
						log.WithError(err).Warn("Failed to unmarshal finalized checkpoint event data")
						continue
					}
					select {
					case <-ctx.Done():
						return
					case eventsCh <- event:
					}
				}
			}
		}
	}()

	return eventsCh, errCh, nil
}
