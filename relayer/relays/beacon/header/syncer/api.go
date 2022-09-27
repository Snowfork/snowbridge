package syncer

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
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
	GetFinalizedHeader() (BeaconHeader, error)
	GetHeadHeader() (BeaconHeader, error)
	GetHeader(id string) (BeaconHeader, error)
	GetSyncCommitteePeriodUpdate(from uint64) (SyncCommitteePeriodUpdateResponse, error)
	GetBeaconBlock(slot uint64) (BeaconBlockResponse, error)
	GetCurrentForkVersion(slot uint64) (string, error)
	GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error)
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

type HeaderResponse struct {
	Slot          string `json:"slot"`
	ProposerIndex string `json:"proposer_index"`
	ParentRoot    string `json:"parent_root"`
	StateRoot     string `json:"state_root"`
	BodyRoot      string `json:"body_root"`
}

type SyncCommitteeResponse struct {
	Pubkeys         []string `json:"pubkeys"`
	AggregatePubkey string   `json:"aggregate_pubkey"`
}

type BranchResponse []string

type BeaconHeaderResponse struct {
	Data struct {
		Root      string `json:"root"`
		Canonical bool   `json:"canonical"`
		Header    struct {
			Message   HeaderResponse `json:"message"`
			Signature string         `json:"signature"`
		} `json:"header"`
	} `json:"data"`
}

type SyncAggregateResponse struct {
	SyncCommitteeBits      string `json:"sync_committee_bits"`
	SyncCommitteeSignature string `json:"sync_committee_signature"`
}

type BeaconHeader struct {
	Slot          uint64
	ProposerIndex uint64
	ParentRoot    common.Hash
	StateRoot     common.Hash
	BodyRoot      common.Hash
}

func (b *BeaconClient) GetFinalizedHeader() (BeaconHeader, error) {
	return b.GetHeader("finalized")
}

func (b *BeaconClient) GetHeadHeader() (BeaconHeader, error) {
	return b.GetHeader("head")
}

func (b *BeaconClient) GetHeader(id string) (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, b.endpoint+"/eth/v1/beacon/headers/"+id, nil)
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

	slot, err := strconv.ParseUint(response.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := strconv.ParseUint(response.Data.Header.Message.ProposerIndex, 10, 64)
	if err != nil {
		return BeaconHeader{}, fmt.Errorf("parse proposerIndex as int: %w", err)
	}

	return BeaconHeader{
		Slot:          slot,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(response.Data.Header.Message.ParentRoot),
		StateRoot:     common.HexToHash(response.Data.Header.Message.StateRoot),
		BodyRoot:      common.HexToHash(response.Data.Header.Message.BodyRoot),
	}, nil
}

type SignedHeaderResponse struct {
	Message   HeaderResponse `json:"message"`
	Signature []byte         `json:"signature"`
}

type CheckpointResponse struct {
	Epoch string `json:"epoch"`
	Root  string `json:"root"`
}

type DepositDataResponse struct {
	Pubkey                string `json:"pubkey"`
	WithdrawalCredentials string `json:"withdrawal_credentials"`
	Amount                string `json:"amount"`
	Signature             string `json:"signature"`
}

type DepositResponse struct {
	Proof []string            `json:"proof"`
	Data  DepositDataResponse `json:"data"`
}

type AttestationDataResponse struct {
	Slot            string             `json:"slot"`
	Index           string             `json:"index"`
	BeaconBlockRoot string             `json:"beacon_block_root"`
	Source          CheckpointResponse `json:"source"`
	Target          CheckpointResponse `json:"target"`
}

type IndexedAttestationResponse struct {
	AttestingIndices []string                `json:"attesting_indices"`
	Data             AttestationDataResponse `json:"data"`
	Signature        string                  `json:"signature"`
}

type AttesterSlashingResponse struct {
	Attestation1 IndexedAttestationResponse `json:"attestation_1"`
	Attestation2 IndexedAttestationResponse `json:"attestation_2"`
}

type ProposerSlashingResponse struct {
	SignedHeader1 SignedHeaderResponse `json:"signed_header_1"`
	SignedHeader2 SignedHeaderResponse `json:"signed_header_2"`
}

type AttestationResponse struct {
	AggregationBits string                  `json:"aggregation_bits"`
	Data            AttestationDataResponse `json:"data"`
	Signature       string                  `json:"signature"`
}

type VoluntaryExitResponse struct {
	Epoch          string `json:"epoch"`
	ValidatorIndex string `json:"validator_index"`
}

type ErrorMessage struct {
	StatusCode int    `json:"statusCode"`
	Error      string `json:"error"`
	Message    string `json:"message"`
}

type BeaconBlockResponse struct {
	Data struct {
		Message struct {
			Slot          string `json:"slot"`
			ProposerIndex string `json:"proposer_index"`
			ParentRoot    string `json:"parent_root"`
			StateRoot     string `json:"state_root"`
			Body          struct {
				RandaoReveal string `json:"randao_reveal"`
				Eth1Data     struct {
					DepositRoot  string `json:"deposit_root"`
					DepositCount string `json:"deposit_count"`
					BlockHash    string `json:"block_hash"`
				} `json:"eth1_data"`
				Graffiti          string                     `json:"graffiti"`
				ProposerSlashings []ProposerSlashingResponse `json:"proposer_slashings"`
				AttesterSlashings []AttesterSlashingResponse `json:"attester_slashings"`
				Attestations      []AttestationResponse      `json:"attestations"`
				Deposits          []DepositResponse          `json:"deposits"`
				VoluntaryExits    []VoluntaryExitResponse    `json:"voluntary_exits"`
				SyncAggregate     SyncAggregateResponse      `json:"sync_aggregate"`
				ExecutionPayload  struct {
					ParentHash    string   `json:"parent_hash"`
					FeeRecipient  string   `json:"fee_recipient"`
					StateRoot     string   `json:"state_root"`
					ReceiptsRoot  string   `json:"receipts_root"`
					LogsBloom     string   `json:"logs_bloom"`
					PrevRandao    string   `json:"prev_randao"`
					BlockNumber   string   `json:"block_number"`
					GasLimit      string   `json:"gas_limit"`
					GasUsed       string   `json:"gas_used"`
					Timestamp     string   `json:"timestamp"`
					ExtraData     string   `json:"extra_data"`
					BaseFeePerGas string   `json:"base_fee_per_gas"`
					BlockHash     string   `json:"block_hash"`
					Transactions  []string `json:"transactions"`
				} `json:"execution_payload"`
			} `json:"body"`
		} `json:"message"`
	} `json:"data"`
}

type BeaconBlock struct {
	Slot          uint64
	ProposerIndex uint64
	ParentRoot    common.Hash
	StateRoot     common.Hash
	BodyRoot      common.Hash
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

type SyncCommitteePeriodUpdateResponse struct {
	Data []struct {
		AttestedHeader          HeaderResponse        `json:"attested_header"`
		NextSyncCommittee       SyncCommitteeResponse `json:"next_sync_committee"`
		NextSyncCommitteeBranch []common.Hash         `json:"next_sync_committee_branch"`
		FinalizedHeader         HeaderResponse        `json:"finalized_header"`
		FinalityBranch          []common.Hash         `json:"finality_branch"`
		SyncAggregate           SyncAggregateResponse `json:"sync_aggregate"`
		ForkVersion             string                `json:"fork_version"`
	} `json:"data"`
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

	var response SyncCommitteePeriodUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return SyncCommitteePeriodUpdateResponse{}, fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response, nil
}

type ForkResponse struct {
	Data struct {
		PreviousVersion string `json:"previous_version"`
		CurrentVersion  string `json:"current_version"`
		Epoch           string `json:"epoch"`
	} `json:"data"`
}

func (b *BeaconClient) GetCurrentForkVersion(slot uint64) (string, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/head/fork", b.endpoint), nil)
	if err != nil {
		return "", fmt.Errorf("%s: %w", ConstructRequestErrorMessage, err)
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("%s: %w", DoHTTPRequestErrorMessage, err)
	}

	if res.StatusCode != http.StatusOK {
		return "", fmt.Errorf("%s: %d", DoHTTPRequestErrorMessage, res.StatusCode)
	}

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		return "", fmt.Errorf("%s: %w", ReadResponseBodyErrorMessage, err)
	}

	var response ForkResponse

	err = json.Unmarshal(bodyBytes, &response)
	if err != nil {
		return "", fmt.Errorf("%s: %w", UnmarshalBodyErrorMessage, err)
	}

	return response.Data.CurrentVersion, nil
}

type LatestFinalisedUpdateResponse struct {
	Data struct {
		AttestedHeader  HeaderResponse        `json:"attested_header"`
		FinalizedHeader HeaderResponse        `json:"finalized_header"`
		FinalityBranch  []common.Hash         `json:"finality_branch"`
		SyncAggregate   SyncAggregateResponse `json:"sync_aggregate"`
		ForkVersion     string                `json:"fork_version"`
	} `json:"data"`
}

func (b *BeaconClient) GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/light_client/finality_update/", b.endpoint), nil)
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

type LatestHeaderUpdateResponse struct {
	Data struct {
		AttestedHeader HeaderResponse        `json:"attested_header"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
	} `json:"data"`
}
