package syncer

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

type BeaconClientTracker interface {
	GetFinalizedHeader() (BeaconHeader, error)
	GetHeadHeader() (BeaconHeader, error)
	GetHeader(id string) (BeaconHeader, error)
	GetSyncCommitteePeriodUpdate(from, to uint64) (SyncCommitteePeriodUpdateResponse, error)
	GetHeadCheckpoint() (FinalizedCheckpointResponse, error)
	GetLightClientSnapshot(blockRoot string) (LightClientSnapshotResponse, error)
	GetTrustedLightClientSnapshot() (LightClientSnapshotResponse, error)
	GetBeaconBlock(slot uint64) (BeaconBlockResponse, error)
	GetGenesis() (GenesisResponse, error)
	GetCurrentForkVersion(slot uint64) (string, error)
	GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error)
	GetLatestHeadUpdate() (LatestFinalisedUpdateResponse, error)
}

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
		logrus.WithError(err).Error("unable to construct beacon header request")

		return BeaconHeader{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconHeader{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return BeaconHeader{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return BeaconHeader{}, nil
	}

	var response BeaconHeaderResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon header json response")

		return BeaconHeader{}, nil
	}

	slot, err := strconv.ParseUint(response.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, nil
	}

	proposerIndex, err := strconv.ParseUint(response.Data.Header.Message.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, nil
	}

	return BeaconHeader{
		Slot:          slot,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(response.Data.Header.Message.ParentRoot),
		StateRoot:     common.HexToHash(response.Data.Header.Message.StateRoot),
		BodyRoot:      common.HexToHash(response.Data.Header.Message.BodyRoot),
	}, nil
}

type BeaconBlockResponse struct {
	Data struct {
		Message struct {
			Slot          string `json:"slot"`
			ProposerIndex string `json:"proposer_index"`
			ParentRoot    string `json:"parent_root"`
			StateRoot     string `json:"state_root"`
			Body          struct {
				ExecutionPayload struct {
					BlockHash string `json:"block_hash"`
				} `json:"execution_payload"`
				SyncAggregate SyncAggregateResponse `json:"sync_aggregate"`
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

func (b *BeaconClient) GetBeaconBlock(slot uint64) (BeaconBlockResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", b.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return BeaconBlockResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconBlockResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return BeaconBlockResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return BeaconBlockResponse{}, nil
	}

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block json response")

		return BeaconBlockResponse{}, nil
	}

	logrus.WithField("sync agg", response).Info("sync agg")

	return response, nil
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

func (b *BeaconClient) GetSyncCommitteePeriodUpdate(from, to uint64) (SyncCommitteePeriodUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/committee_updates?from=%d&to=%d", b.endpoint, from, to), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	//logrus.WithFields(logrus.Fields{"body": string(bodyBytes), "period": from}).Info("snapshot")

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	var response SyncCommitteePeriodUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal sync committee update json response")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	return response, nil
}

type SyncCommitteeIndexesResponse struct {
	Data struct {
		Validators []string `json:"validators"`
	} `json:"data"`
}

type SyncCommitteeIndexes struct {
	Indexes []uint64
}

func (b *BeaconClient) GetSyncCommittee(epoch uint64) (SyncCommitteeIndexes, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/sync_committees?epoch=%v", b.endpoint, epoch), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct sync committee request")

		return SyncCommitteeIndexes{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncCommitteeIndexes{}, nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return SyncCommitteeIndexes{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncCommitteeIndexes{}, nil
	}

	var response SyncCommitteeIndexesResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal sync committee json response")

		return SyncCommitteeIndexes{}, nil
	}

	syncCommittee := SyncCommitteeIndexes{
		Indexes: []uint64{},
	}

	for _, validatorIndex := range response.Data.Validators {
		index, err := strconv.ParseUint(validatorIndex, 10, 64)
		if err != nil {
			logrus.WithError(err).Error("unable parse slot as int")

			return SyncCommitteeIndexes{}, nil
		}

		syncCommittee.Indexes = append(syncCommittee.Indexes, index)
	}

	return syncCommittee, nil
}

type ForkResponse struct {
	Data struct {
		PreviousVersion string `json:"previous_version"`
		CurrentVersion  string `json:"current_version"`
		Epoch           string `json:"epoch"`
	} `json:"data"`
}

func (b *BeaconClient) GetCurrentForkVersion(slot uint64) (string, error) {
	//req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/%d/fork", s.endpoint, slot), nil)
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/fork", b.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct fork version request")

		return "", nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return "", nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return "", nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return "", nil
	}

	var response ForkResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return "", nil
	}

	return response.Data.CurrentVersion, nil
}

type FinalizedCheckpointResponse struct {
	Data struct {
		PreviousJustified struct {
			Epoch string `json:"epoch"`
			Root  string `json:"root"`
		} `json:"previous_justified"`
		CurrentJustified struct {
			Epoch string `json:"epoch"`
			Root  string `json:"root"`
		} `json:"current_justified"`
		Finalized struct {
			Epoch string `json:"epoch"`
			Root  string `json:"root"`
		} `json:"finalized"`
	} `json:"data"`
}

func (b *BeaconClient) GetFinalizedCheckpoint() (FinalizedCheckpointResponse, error) {
	return b.GetCheckpoint("finalized")
}

func (b *BeaconClient) GetHeadCheckpoint() (FinalizedCheckpointResponse, error) {
	return b.GetCheckpoint("head")
}

func (b *BeaconClient) GetCheckpoint(state string) (FinalizedCheckpointResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/%s/finality_checkpoints", b.endpoint, state), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct finalized checkpoint request")

		return FinalizedCheckpointResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return FinalizedCheckpointResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return FinalizedCheckpointResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return FinalizedCheckpointResponse{}, nil
	}

	var response FinalizedCheckpointResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return FinalizedCheckpointResponse{}, nil
	}

	return response, nil
}

type LightClientSnapshotData struct {
	Header                     HeaderResponse        `json:"header"`
	CurrentSyncCommittee       SyncCommitteeResponse `json:"current_sync_committee"`
	CurrentSyncCommitteeBranch []common.Hash         `json:"current_sync_committee_branch"`
}

type LightClientSnapshotResponse struct {
	Data LightClientSnapshotData `json:"data"`
}

func (b *BeaconClient) GetLightClientSnapshot(blockRoot string) (LightClientSnapshotResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/snapshot/%s", b.endpoint, blockRoot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct light client snapshot request")

		return LightClientSnapshotResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LightClientSnapshotResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LightClientSnapshotResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LightClientSnapshotResponse{}, nil
	}

	var response LightClientSnapshotResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal light client snapshot json response")

		return LightClientSnapshotResponse{}, nil
	}

	//logrus.WithFields(logrus.Fields{"body": response}).Info("snapshot")

	return response, nil
}

type GenesisResponse struct {
	Data struct {
		ValidatorsRoot string `json:"genesis_validators_root"`
		Time           string `json:"genesis_time"`
		ForkVersion    string `json:"genesis_fork_version"`
	} `json:"data"`
}

func (b *BeaconClient) GetGenesis() (GenesisResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/genesis", b.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct genesis request")

		return GenesisResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return GenesisResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return GenesisResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return GenesisResponse{}, nil
	}

	var response GenesisResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal genesis json response")

		return GenesisResponse{}, nil
	}

	return response, nil
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/latest_finalized_head_update/", b.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct latest finalized header update request")

		return LatestFinalisedUpdateResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LatestFinalisedUpdateResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LatestFinalisedUpdateResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LatestFinalisedUpdateResponse{}, nil
	}

	var response LatestFinalisedUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal genesis json response")

		return LatestFinalisedUpdateResponse{}, nil
	}

	return response, nil
}

type LatestHeaderUpdateResponse struct {
	Data struct {
		AttestedHeader HeaderResponse        `json:"attested_header"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
	} `json:"data"`
}

func (b *BeaconClient) GetLatestHeadUpdate() (LatestHeaderUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/latest_head_update/", b.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct latest head update request")

		return LatestHeaderUpdateResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LatestHeaderUpdateResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LatestHeaderUpdateResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LatestHeaderUpdateResponse{}, nil
	}

	var response LatestHeaderUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal genesis json response")

		return LatestHeaderUpdateResponse{}, nil
	}

	return response, nil
}
