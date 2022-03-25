package syncer

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

const SLOTS_IN_EPOCH uint64 = 32

const EPOCHS_IN_PERIOD uint64 = 512

type Syncer interface {
	GetHeader() error
}

type Sync struct {
	httpClient http.Client
	endpoint   string
}

func New(endpoint string) Sync {
	return Sync{
		http.Client{},
		endpoint,
	}
}

type BeaconHeaderResponse struct {
	Data struct {
		Root      string `json:"root"`
		Canonical bool   `json:"canonical"`
		Header    struct {
			Message struct {
				Slot          string `json:"slot"`
				ProposerIndex string `json:"proposer_index"`
				ParentRoot    string `json:"parent_root"`
				StateRoot     string `json:"state_root"`
				BodyRoot      string `json:"body_root"`
			} `json:"message"`
			Signature string `json:"signature"`
		} `json:"header"`
	} `json:"data"`
}

type BeaconHeader struct {
	Slot          uint64
	ProposerIndex uint64
	ParentRoot    common.Hash
	StateRoot     common.Hash
	BodyRoot      common.Hash
}

func (s *Sync) GetFinalizedHeader() (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/headers/finalized", nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon header request")

		return BeaconHeader{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return BeaconHeader{}, nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

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
				Eth1Data struct {
					DepositRoot  string `json:"deposit_root"`
					DepositCount string `json:"deposit_count"`
					BlockHash    string `json:"block_hash"`
				} `json:"eth1_data"`
				SyncAggregate struct {
					SyncCommitteeBits      string `json:"sync_committee_bits"`
					SyncCommitteeSignature string `json:"sync_committee_signature"`
				} `json:"sync_aggregate"`
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

type SyncAggregate struct {
	SyncCommitteeBits      string
	SyncCommitteeSignature string
}

func (s *Sync) GetBlockSyncAggregate(slot uint64) (SyncAggregate, error) {
	//req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/blocks/finalized", nil)
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", s.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return SyncAggregate{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncAggregate{}, nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return SyncAggregate{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncAggregate{}, nil
	}

	//logrus.WithFields(logrus.Fields{"body": string(bodyBytes)}).Info("block response")

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block json response")

		return SyncAggregate{}, nil
	}

	logrus.WithField("sync agg", response).Info("sync agg")

	return SyncAggregate{
		SyncCommitteeBits:      HexToBinaryString(response.Data.Message.Body.SyncAggregate.SyncCommitteeBits),
		SyncCommitteeSignature: response.Data.Message.Body.SyncAggregate.SyncCommitteeSignature,
	}, nil
}

type SyncCommitteeResponse struct {
	Data struct {
		Validators []string `json:"validators"`
	} `json:"data"`
}

type SyncCommittee struct {
	Indexes []uint64
}

func (s *Sync) GetSyncCommittee(epoch uint64) (SyncCommittee, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/sync_committees?epoch=%v", s.endpoint, epoch), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct sync committee request")

		return SyncCommittee{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return SyncCommittee{}, nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return SyncCommittee{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return SyncCommittee{}, nil
	}

	var response SyncCommitteeResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal sync committee json response")

		return SyncCommittee{}, nil
	}

	syncCommittee := SyncCommittee{
		Indexes: []uint64{},
	}

	for _, validatorIndex := range response.Data.Validators {
		index, err := strconv.ParseUint(validatorIndex, 10, 64)
		if err != nil {
			logrus.WithError(err).Error("unable parse slot as int")

			return SyncCommittee{}, nil
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

func (s *Sync) GetPubforkVersion(slot uint64) (string, error) {
	//req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/%d/fork", s.endpoint, slot), nil)
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/fork", s.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct fork version request")

		return "", nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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

func (s *Sync) GetFinalizedCheckpoint() (FinalizedCheckpointResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/finality_checkpoints", s.endpoint), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct finalized checkpoint request")

		return FinalizedCheckpointResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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

type LightSnapshotResponse struct {
	Data struct {
		Header struct {
			Slot          string `json:"slot"`
			ProposerIndex string `json:"proposer_index"`
			ParentRoot    string `json:"parent_root"`
			StateRoot     string `json:"state_root"`
			BodyRoot      string `json:"body_root"`
		} `json:"header"`
		CurrentSyncCommittee struct {
			Pubkeys          []string `json:"pubkeys"`
			AggregatePubkeys string   `json:"aggregate_pubkey"`
		} `json:"current_sync_committee"`
		CurrentSyncCommitteeBranch []string `json:"current_sync_committee_branch"`
	} `json:"data"`
}

func (s *Sync) GetLightClientSnapshot(blockRoot string) (LightSnapshotResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/snapshot/%s", s.endpoint, blockRoot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct light client snapshot request")

		return LightSnapshotResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return LightSnapshotResponse{}, nil
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return LightSnapshotResponse{}, nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return LightSnapshotResponse{}, nil
	}

	var response LightSnapshotResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return LightSnapshotResponse{}, nil
	}

	//logrus.WithFields(logrus.Fields{"body": response}).Info("snapshot")

	return response, nil
}

func (l LightSnapshotResponse) ToBeaconHeader() (BeaconHeader, error) {
	slot, err := strconv.ParseUint(l.Data.Header.Slot, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, nil
	}

	proposerIndex, err := strconv.ParseUint(l.Data.Header.ProposerIndex, 10, 64)
	if err != nil {
		logrus.WithError(err).Error("unable parse slot as int")

		return BeaconHeader{}, nil
	}

	return BeaconHeader{
		Slot:          slot,
		ProposerIndex: proposerIndex,
		ParentRoot:    common.HexToHash(l.Data.Header.ParentRoot),
		StateRoot:     common.HexToHash(l.Data.Header.StateRoot),
		BodyRoot:      common.HexToHash(l.Data.Header.BodyRoot),
	}, nil
}

func ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func ComputeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EPOCHS_IN_PERIOD - (epoch % EPOCHS_IN_PERIOD))
}

func HexToBinaryString(rawHex string) string {
	hex := strings.Replace(rawHex, "0x", "", -1)

	// Chunkify strings into array of strings of 8 characters long (to ParseUint safely below)
	chunkSize := 8

	resultStr := ""
	chunks := []string{}
	for i, r := range hex {
		resultStr = resultStr + string(r)
		if i > 0 && (i+1)%chunkSize == 0 {
			chunks = append(chunks, resultStr)
			resultStr = ""
		}
	}

	// If there was a remainder, add the last string to the chunks as well.
	if resultStr != "" {
		chunks = append(chunks, resultStr)
	}

	// Convert chunks into binary string
	binaryStr := ""
	for _, str := range chunks {
		i, err := strconv.ParseUint(str, 16, 32)
		if err != nil {
			fmt.Printf("%s", err)
		}
		binaryStr = binaryStr + fmt.Sprintf("%b", i)
	}

	return binaryStr
}
