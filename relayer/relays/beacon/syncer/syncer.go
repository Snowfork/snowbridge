package syncer

import (
	"encoding/json"
	"io"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

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

func (s *Sync) GetFinalizedHeader() (BeaconHeader, error) {
	client := &http.Client{}

	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/headers/finalized", nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon header request")

		return BeaconHeader{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := client.Do(req)
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

func (s *Sync) GetBlockSyncAggregate() (SyncAggregate, error) {
	client := &http.Client{}

	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/blocks/finalized", nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return SyncAggregate{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := client.Do(req)
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

	logrus.WithField("body", string(bodyBytes)).Info("body")

	var response BeaconBlockResponse

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal beacon block json response")

		return SyncAggregate{}, nil
	}

	logrus.WithField("sync agg", response).Info("sync agg")

	bytes := common.Hex2BytesFixed(response.Data.Message.Body.SyncAggregate.SyncCommitteeBits, 512)

	return SyncAggregate{
		SyncCommitteeBits:      bytes,
		SyncCommitteeSignature: response.Data.Message.Body.SyncAggregate.SyncCommitteeSignature,
	}, nil
}

func (s *Sync) GetSyncCommittee() (SyncCommittee, error) {
	client := &http.Client{}

	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/states/finalized/sync_committees", nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct sync committee request")

		return SyncCommittee{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := client.Do(req)
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
