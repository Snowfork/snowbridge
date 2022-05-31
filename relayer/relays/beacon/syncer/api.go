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
	GetBeaconBlockBySlot(slot uint64) (BeaconBlockResponse, error)
	GetGenesis() (GenesisResponse, error)
	GetCurrentForkVersion(slot uint64) (string, error)
	GetLatestFinalizedUpdate() (LatestFinalisedUpdateResponse, error)
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

		return BeaconHeader{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconHeader{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return BeaconHeader{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return BeaconHeader{}, err
	}

	var response BeaconHeaderResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon header json response")

		return BeaconHeader{}, err
	}

	slot, err := strconv.ParseUint(response.Data.Header.Message.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, err
	}

	proposerIndex, err := strconv.ParseUint(response.Data.Header.Message.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, err
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
	Pubkey                []byte `json:"pubkey"`
	WithdrawalCredentials string `json:"withdrawal_credentials"`
	Amount                string `json:"amount"`
	Signature             []byte `json:"signature"`
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
	Signature        []byte                  `json:"signature"`
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
	AggregationBits []byte                  `json:"aggregation_bits"`
	Data            AttestationDataResponse `json:"data"`
	Signature       []byte                  `json:"signature"`
}

type VoluntaryExitResponse struct {
	Epoch          string `json:"epoch"`
	ValidatorIndex string `json:"validator_index"`
}

type BeaconBlockResponse struct {
	Data struct {
		Message struct {
			Slot          string `json:"slot"`
			ProposerIndex string `json:"proposer_index"`
			ParentRoot    string `json:"parent_root"`
			StateRoot     string `json:"state_root"`
			Body          struct {
				RandaoReveal []byte `json:"randao_reveal"`
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
					ParentHash    string
					FeeRecipient  []byte
					StateRoot     string
					ReceiptsRoot  string
					LogsBloom     []byte
					PrevRandao    string
					BlockNumber   string
					GasLimit      string
					GasUsed       string
					Timestamp     string
					ExtraData     []byte
					BaseFeePerGas string
					BlockHash     string
					Transactions  [][]byte
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
		logrus.WithError(err).Error("unable to construct beacon block request")

		return BeaconBlockResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconBlockResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return BeaconBlockResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return BeaconBlockResponse{}, err
	}

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block json response")

		return BeaconBlockResponse{}, err
	}

	return response, nil
}

func (b *BeaconClient) GetBeaconBlockBySlot(slot uint64) (BeaconBlockResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", b.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return BeaconBlockResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconBlockResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return BeaconBlockResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return BeaconBlockResponse{}, err
	}

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block json response")

		return BeaconBlockResponse{}, err
	}

	return response, nil
}

func (b *BeaconClient) GetBeaconBlockRoot(slot uint64) (common.Hash, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/blocks/%d/root", b.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return common.Hash{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return common.Hash{}, err
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return common.Hash{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return common.Hash{}, err
	}

	var response struct {
		Data string `json:"data"`
	}

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block root json response")

		return common.Hash{}, err
	}

	return common.HexToHash(response.Data), nil
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

		return SyncCommitteePeriodUpdateResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncCommitteePeriodUpdateResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return SyncCommitteePeriodUpdateResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	//logrus.WithFields(logrus.Fields{"body": string(bodyBytes), "period": from}).Info("snapshot")

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncCommitteePeriodUpdateResponse{}, err
	}

	var response SyncCommitteePeriodUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal sync committee update json response")

		return SyncCommitteePeriodUpdateResponse{}, err
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

		return SyncCommitteeIndexes{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncCommitteeIndexes{}, err
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return SyncCommitteeIndexes{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncCommitteeIndexes{}, err
	}

	var response SyncCommitteeIndexesResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal sync committee json response")

		return SyncCommitteeIndexes{}, err
	}

	syncCommittee := SyncCommitteeIndexes{
		Indexes: []uint64{},
	}

	for _, validatorIndex := range response.Data.Validators {
		index, err := strconv.ParseUint(validatorIndex, 10, 64)
		if err != nil {
			logrus.WithError(err).Error("unable parse slot as int")

			return SyncCommitteeIndexes{}, err
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
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/%d/fork", b.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct fork version request")

		return "", err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return "", err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return "", err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return "", err
	}

	var response ForkResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return "", err
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

		return FinalizedCheckpointResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return FinalizedCheckpointResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return FinalizedCheckpointResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return FinalizedCheckpointResponse{}, err
	}

	var response FinalizedCheckpointResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return FinalizedCheckpointResponse{}, err
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

		return LightClientSnapshotResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LightClientSnapshotResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LightClientSnapshotResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LightClientSnapshotResponse{}, err
	}

	var response LightClientSnapshotResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal light client snapshot json response")

		return LightClientSnapshotResponse{}, err
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

		return GenesisResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return GenesisResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return GenesisResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return GenesisResponse{}, err
	}

	var response GenesisResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal genesis json response")

		return GenesisResponse{}, err
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

		return LatestFinalisedUpdateResponse{}, err
	}

	req.Header.Set("accept", "application/json")
	res, err := b.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LatestFinalisedUpdateResponse{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LatestFinalisedUpdateResponse{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LatestFinalisedUpdateResponse{}, err
	}

	var response LatestFinalisedUpdateResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal genesis json response")

		return LatestFinalisedUpdateResponse{}, err
	}

	return response, nil
}

type LatestHeaderUpdateResponse struct {
	Data struct {
		AttestedHeader HeaderResponse        `json:"attested_header"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
	} `json:"data"`
}
