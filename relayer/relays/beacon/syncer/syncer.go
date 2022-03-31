package syncer

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

const SLOTS_IN_EPOCH uint64 = 32

const EPOCHS_PER_SYNC_COMMITTEE_PERIOD uint64 = 256

type Syncer interface {
	GetFinalizedHeader() (BeaconHeader, error)
	GetHeadHeader() (BeaconHeader, error)
	GetHeader(id string) (BeaconHeader, error)
	GetSyncCommitteePeriodUpdate(from, to uint64) (SyncCommitteePeriodUpdateResponse, error)
	GetHeadCheckpoint() (FinalizedCheckpointResponse, error)
	GetLightClientSnapshot(blockRoot string) (LightClientSnapshotResponse, error)
	GetTrustedLightClientSnapshot() (LightClientSnapshotResponse, error)
	GetBeaconBlock(slot uint64) (BeaconBlockResponse, error)
}

type Sync struct {
	httpClient http.Client
	endpoint   string
}

func New(endpoint string) *Sync {
	return &Sync{
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

func (s *Sync) GetFinalizedHeader() (BeaconHeader, error) {
	return s.GetHeader("finalized")
}

func (s *Sync) GetHeadHeader() (BeaconHeader, error) {
	return s.GetHeader("head")
}

func (s *Sync) GetHeader(id string) (BeaconHeader, error) {
	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/headers/"+id, nil)
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
				Eth1Data struct {
					DepositRoot  string `json:"deposit_root"`
					DepositCount string `json:"deposit_count"`
					BlockHash    string `json:"block_hash"`
				} `json:"eth1_data"`
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

func (s *Sync) GetBeaconBlock(slot uint64) (BeaconBlockResponse, error) {
	//req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/blocks/finalized", nil)
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v2/beacon/blocks/%d", s.endpoint, slot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return BeaconBlockResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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

	//logrus.WithFields(logrus.Fields{"body": string(bodyBytes)}).Info("block response")

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
		NextSyncCommitteeBranch HeaderResponse        `json:"next_sync_committee_branch"`
		FinalizedHeader         HeaderResponse        `json:"finalized_header"`
		FinalityBranch          HeaderResponse        `json:"finality_branch"`
		SyncAggregate           SyncAggregateResponse `json:"sync_committee_aggregate"`
		ForkVersion             string                `json:"fork_version"`
	} `json:"data"`
}

func (s *Sync) GetSyncCommitteePeriodUpdate(from, to uint64) (SyncCommitteePeriodUpdateResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/committee_updates?from=%d&to=%d", s.endpoint, from, to), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct beacon block request")

		return SyncCommitteePeriodUpdateResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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

func (s *Sync) GetSyncCommittee(epoch uint64) (SyncCommitteeIndexes, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/finalized/sync_committees?epoch=%v", s.endpoint, epoch), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct sync committee request")

		return SyncCommitteeIndexes{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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
	return s.GetCheckpoint("finalized")
}

func (s *Sync) GetHeadCheckpoint() (FinalizedCheckpointResponse, error) {
	return s.GetCheckpoint("head")
}

func (s *Sync) GetCheckpoint(state string) (FinalizedCheckpointResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/beacon/states/%s/finality_checkpoints", s.endpoint, state), nil)
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

type LightClientSnapshotData struct {
	Header                     HeaderResponse        `json:"header"`
	CurrentSyncCommittee       SyncCommitteeResponse `json:"current_sync_committee"`
	CurrentSyncCommitteeBranch []string              `json:"current_sync_committee_branch"`
}

type LightClientSnapshotResponse struct {
	Data LightClientSnapshotData `json:"data"`
}

func (s *Sync) GetLightClientSnapshot(blockRoot string) (LightClientSnapshotResponse, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/eth/v1/lightclient/snapshot/%s", s.endpoint, blockRoot), nil)
	if err != nil {
		logrus.WithError(err).Error("unable to construct light client snapshot request")

		return LightClientSnapshotResponse{}, nil
	}

	req.Header.Set("accept", "application/json")
	res, err := s.httpClient.Do(req)
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
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return LightClientSnapshotResponse{}, nil
	}

	//logrus.WithFields(logrus.Fields{"body": response}).Info("snapshot")

	return response, nil
}

func (s *Sync) GetTrustedLightClientSnapshot() (LightClientSnapshotResponse, error) {
	return LightClientSnapshotResponse{
		Data: LightClientSnapshotData{
			Header: HeaderResponse{
				Slot:          "3476320",
				ProposerIndex: "168760",
				ParentRoot:    "0x244aca04180a684f0af2f18e47e86c55f72e28ec8c1962d538b1b2490af0fbb0",
				StateRoot:     "0xc1fbb5f95ce267fe2da3ac96d177558b1a31c27bce5b8aa6d07ffb5233f0fd55",
				BodyRoot:      "0xdb5e2bb7ef80f1b7ee7d464ccf364ad0263ca8b30580bad6d2893e655f7dc2af",
			},
			CurrentSyncCommittee: SyncCommitteeResponse{
				Pubkeys: []string{
					"0x883f0aba4782021304a10a3ce63f8e1d2c31f497ef573eebbabc12538c5aeac6317a4537258963153e47536637d058c0",
					"0xb9b0c8bbabeac871cd5712aa373751966717c73fe62b511bec7348a5454c4a3b7c6136d7b737cc945cdc216895650cfe",
					"0xa5ae35ed4c9eaaf48bada09b59b69e33e3a4ff4ec0042fa501a3d4c2916dcce1746ed7cb21e9936ac47ebe70101b17da",
					"0xb341041be1578bce5dc449f43d45d3815d61108f7d2fa723faa41124dbbd2b733388cad692e9e8b16467ab899952306b",
					"0xb24ca505ec64fcd7c4bcf5c3524ade5fde273ad1bc31047dfa3bfe647c3884bd7ed48a1d0f13250eae42085f5db9f799",
					"0xb6a7311df6badd20d44896fa50e84c7805f8c724373e11f55ac15d87d5d30622a2bb1b4b580b8846db33b6c4e0e6549e",
					"0x98143706ae63936ba217808aa0fb373278ffdcc90cab79cf7dd010606fab1f97cb9e8ff19e20c3d44acbcecaa8ce42e4",
					"0x81ed1ec6311466ec073b0838597682adcb25161839e55d0446cf349601cc6aa990436cd9959ce089b7ff6d68879cd7e3",
					"0x8fcdbc3ff1dc7f64502c7f9b2d29f20fb78af8927289f7ac5265af6ecc43a8b4241602b0808281bb2a5c9c111c2af056",
					"0xb3a2b7bfbef8f6c42efb8f2680528cf90b11e75ff033123b5a7ac0239245f6e9c8e2e732e8efbae26acdac0855167d33",
					"0xa011703c3ec6c404587956248b820976d9b1ac453abdcba3001adc9b5b2f1d0814284bb5f26000aedab7d0920f246aeb",
					"0x9930171ac0a62a5a0270030e2b552c2155ba9cc867cf58c86d64653fea08a891fa64568f07fc8405b050b48d43b0721b",
					"0x9962489628b530edca92989e9b77dcc3c0dfe93c254c2fe6f2a1b91db7e9424f3bac613938e67f7bd9c2374008ed3de9",
					"0xb8c1701f8ca1f0a9aa8455c32fcc4cfbde141ce21575067d0b7808d1813821b2bad47825ed28b222ad40f22d93867c7c",
					"0xa583f609f63c36b04796f05390ed40f3bdf426bd75ba5aff34076baa8725c37e28c69381cb1737db6a7fada6ea0238f8",
					"0x8f05a54dcb4087cece21d1a58d9d5b3c43f476f0845eaffe0239f94821b9f20ad2b8fb74576e44d9f5c5b3ab9571faee",
					"0x86f0dac8f55d94f957200ef408fa89cb92f22b9dda098834378233d28af4169d56be2135a66c9c5a256f0fe905ef3175",
					"0xb653bb6041ccaab5eb326299bf3e7033c21b8817abba53392110269f34eef1d0c335c912b58c2cb14810a5ea3db2db7a",
					"0xa7d6b40b5ef77fc12a7b6b8fd0b271c7c8900f69ab07804d7a4981a7ba2b6e59cef3ff0b716546c7e23b6d03d3b65d52",
					"0xb19a84a47e4b761377dce2692491a962fbe38bd834e92faea76e994565dea8fc1840d5490e1e56eb9cadeb68930fea29",
					"0x9287e758a0ef33168d86bb13825101f254132d2c695a58c859dd72acb0d87519d1322feff5d1052c6e2b80f13b61d8de",
					"0xb85c78ad4997a15b700d46d1afbce7efcdd460b05d0d2162223f109bb3d7430e35ca7a7441ab2ee7a91ba6b133ab19d5",
					"0xb04157dc813168c9396bfb19f9c24af219bf8cc2d1df52be7fdaa11b0f0208e77b9aa81405acdb09ceb1a087d713099e",
					"0xa31ed4b7b7fa2d96e2ea1401613ec917543d7bf17f9f973d3c637b5ac528e8f486a09c94b847fe095819bb0cc2ed5749",
					"0xa650c546cf39444b0ffcdf21196d60a2ce93ac49f6f447d626ec3f9a2d0b76091f9851042572717d0a4412ceed644053",
					"0x977905236f013baee9af665afe93a57236600e1dbf451245d6645375bc830f4853da172c88b5fd9cce88a21d9302b33d",
					"0xb0933ffa12e6e383ca3efd7335f61bcd20d955662a75c237995e9e3021e3485582e53e8d329db170976b5bc39ad7aae2",
					"0xb6daf17ab8c4551c0af8c1f14b564d8363e813d851eb8934603b92a9dfa5c513b360e940e8bd09eef554faf1ec94a7b7",
					"0x85ff5f3be45a37feef521465415ad1d4b3095a8194a17bcfa8c76fb9014b10afa87a0300d1fac273c7dae2ea590c70bf",
					"0x8faebce7c3a68bc59a50278aa4a424ba957af6ad4acb768e90ca46c1a6b81b33fbaa33eb33de1a0bd0b602ed49de17a6",
					"0x88fe225524394094acc62417deb7cc12edc1edd20eb7d928a20ad6d779ed73ceb3e00766c764f2fcad8345e15cd52322",
					"0xa313977295231afe188813f627bccc20a86782bbfcb073c3057e3cb3629f68512fc84f1ebe11a94f3db40239114a0b2a",
					"0x95814c488b59fd8361be5c05cc50bcfb86ac49923b010b8f4237acc11d8fe3e5c0250bc79ea9d3ce0083d91466361c2f",
					"0x990117a1adb3ccad23d78b627a26aaaa041699690d7b37a84a0402eae0e03bcf801034662b63b0c8440fddfcaeb819e4",
					"0xb05b6acca9b63979b353c76c4e3a15c33e2114f3eb181dacd4783e4aa77afd660b13e750f94b26c3f2eaa98fc93ed41a",
					"0xb8097ca29053c6dc62f1f41ee283ffca565c0dda5959ed9d5ccb8725e40d2be7d7a089dd2d0b16a3e8ec18f7414c4ff1",
					"0xa08b2449e70c44dd6ccf9c6f87d86f8d1c5615b4438f7115104d7959821f67d7505070f4874fe1ea200a40a1efd99a72",
					"0xb5790c4b360d6ff155368b7bacfcbb6e0a8c12abd2217ec72432e92520df153f6bf63c39525ae05730a846641befa77c",
					"0x836761fd5f9a83ba5c531b0c50703b47b92b5588a03ac5a6de9283023b48950fade6685f4120d29db2efe655f9d4e5f2",
					"0xb73e43da410eed50dfc634d5d0815829a6cd924522847d88bee7c6664bd6dc5ce08aeacdf48d01857fc794d1f4a8466d",
					"0xb28b0c39b8575efc1c01fb18cb93c8ddd57c724016bed9669d5dcb87f3f50e7fbf21bb3f78145a30caf9619e92ec8f15",
					"0xb64c33609e6faf36b33b32dd2dde814d5515a85506f7707979f28c3029209af57a011e77d80a9570213d599502ff2cfd",
					"0xb2c00320d5a8d8f784ebb677661eb573edd4078a7622ff4cd06f8c431fe8b45f675552ac70cb73c74012a20b0418d1f3",
					"0x9432a757f19825b11a668956a46cc7ef7d0c2f0d35a1559b07edc6815e4613ff91dbfc8ceca4c72714481273a2ca5df2",
					"0x95b69a4c4afd7f294858aae2c01dc7a94bdb602b126cb27eeaac7a0ed1c76c7f667a45a8c6f8d48ec3618b4c2b4fae25",
					"0xa49bc869410535bf46c470b31cd167c3fdf6e83691e804ed1d0d89534dafa4d12ccd2531906118e50925fb1daaac0edb",
					"0x8301475e95df7a2d667ae89fe775f0e24928d5d704896adec30457e28d9f17c44bb99f4cc78876e4b58807b2322ddc21",
					"0x8960594c5d3fc63e55ddd4b148b7b46410549a71bb229ffe5a15d9e30e7e6bcd1b44fc1edef84af4afa81149ba5cdf03",
					"0x81b19566e9fff65e08273eea323cf67a572d9cd554e8d66945aca34a70cea6cca51bd0ae398b28eeaf0269df4d77facd",
					"0xa2c2dce9bfc5000dc152ec16e5c84895d7fe023c9b6fec9b618e6d25672600ac4bdda68abfe95d3f635ce859a956f752",
					"0xb8d3557b0cf196ccb2591f84f7c9021f32df70eaf2dd29f40bc92a29d32293c89bd704fb147a2acf48b1c65c122491a4",
					"0xb9cdc55cc6d4b07cfe8bcde5f88d9fcc3d51ceca13bba8c2aa0e7b1c46fcd7b71a842980ae22873fad3fe79c8a8962c7",
					"0xb2c487448d8572d81dc16aecf41213843d1af4d62aa57b63f8b4ad7c7b523f7432978e4a029cfd86bd0804e63b1b83d9",
					"0x898581607ef065e15ba36aeb530eada499531284426e542c3a307df1722d72122e7846fc3d770c8f475d66cd9d5004be",
					"0xb5d5417549c56e6400b92ab6f3edd02e75e156aea0f266ca4ff926baf88194804bce8572d0495d33c5d2cbca5004d38a",
					"0xa5473f91e1d1254429225ce42bd6db74db72e46ad75a0655810566e6a951599bbbc32a83cd90a81ceb38d7dd9ced0a51",
					"0xa3322e82a2fcc1ff9e7b8272b05fc12b41423e0cd2a1ddd71b050ee5d86d22773ede550ae66a1cb14946f83c9be828ff",
					"0xa4a83a6317dd71943bbd6bcd58df7afeabd5b6af4037f3e6640340ac03b6b6fa15d4d65ae91ffa30a9f2b1eb06a9c897",
					"0xae11ee7801f6997584347ddd01a222a8f4bfd2eab813cbc2672f91579b6f087854fd8d51b71c648c9337d5042a6f34dc",
					"0xad9d28d05fa20123dcb0744d714672f85e0581c10876ba7c7bb827c3f9971826b616799a6d88a005b5c878b09301ac14",
					"0x92f873de7b7c8db563b7e137d6b4cee2937fd23646e6b7a2c28935959aee046c41b0725707e09563852c3f1b4778a868",
					"0xac5255a4981afe8cda8cc9a67d56c3f9ad90c9adaafb5ba467bbd169e55c05898d4980bad70e61eecaebdded2525284a",
					"0xab64cd07b913c6d6b1e79c7b8ed548a3f74f0e439f41e3af30fba9bbee2c01c430337a1aa426ae4e0af9488eda6fde65",
					"0xb716566a75eea83564c0a2061e6f4bdd76639057aed611688164651e59921d0dd43c6176baad344fb0c357d985bb0641",
					"0xab6fcc8cc6f420911a403465aea14940988dc30ccf71ea08a884d8b2091a5b1e0c2ec680e2ab8bcd14e8b39e7a3d2cb2",
					"0x9484c8054eea4c8ab739e7b53cb6deea3284d2fa876317dc17480d99498f54e8005c86b958f410c5ff17009d7cd022d5",
					"0x856e8a7f8b200745e3f20bad160cc92d91f85d596810e1df148916ad9d30dbb411a07f4b3532ebec88c70a8bca03d50b",
					"0xa7267d7e30f3675262bb0f15bb23f8ba9b965846083b57c32b7884567ac33604b5ec83fb6e719f6bfa9ae092c9f465cf",
					"0x80b72104ff54d0a5e4d142698ea2c1edd9bf4c6b0b654c40055773cdec10f3fe21873ecd9f3bf5b1fb9431d551bf235b",
					"0x86645c5d05f05dc3105d7e4383dd77788ccae9642f5555afd9810de0877047737fd26d359953d3abb5a1975904b045bf",
					"0xac9a3fd9e57b8ac22d2df28ebbaff5335ffcbf8dd0e091b73660ee8d6e996ba6c2a182d026d68e867e509ea87cffb9c6",
					"0x84d01e96117d4522134da7ba8815cc70c82f6bb8be10e6668c6a026b2aa87ca517cc99a2d7649f26f0bef8490463669a",
					"0x9321ed8e0ba65356a7a87107fc2e720c056005514919630434919b4b3e767bad16f4fe4e57e048f5f3cb1d50ffb4bc58",
					"0xa843778884f69bc0828260b4d035cc7f46a0e5f029d3d2739a52dc7d25d4f5d6f22c32cbac997241baf4a5e82c7217c5",
					"0x873d8c766111feb9bd70fb30989cf111e89e09d3586de2559f747142949fd0049ca4e0624ce3db183369e81ac03b1381",
					"0xb5b3eb1ba2547da9c7ff4db90985c7099ab8c2dce5da25ace9f3e7353458ec9bf44495cd9d9ea8f53c59090bc4370e3b",
					"0xb97248b1d00057ffae275b3eb51de9c99f0688f29fde92f3b35563154d3f451669ad4ec7ddc9c4015f0483e9941cd88d",
					"0x9199a0e7c1e3b5fadb895f78dc08ad66e238c2fa8842b6064c7e899cce67e307ca3b4ec9c2e1829550f5b79c1db4fdbf",
					"0xb3ad624e001bd9bc855e4f217e1d20b55055af42b87389c0d44df56484ca7b76b57fe1c6e8cfc76e31271282d1195550",
					"0xb771b2168065afa65163c0b28f1b3c097498a1c2a9c3148710de9bf901d39ab03fa1bd6f04d0a668e9ced2147597e279",
					"0x877a356ce0b6a77cff05fba7943513e659dc1b88c936f0b7f4205c043a9a30110ecf77f25958d90d2da8613337efaa30",
					"0x8f1755f98278f027584b5db69a46396829ba403c049adca9c43b7a0a3304ab847aa6aefdcd57be549c85eefc2c1c9f4c",
					"0x821066d39b41bb05490afebfdfbb11fcfb04bc201840ac1033aca31fcf1b284c26f81d829663073ea60a4eb5702963af",
					"0x964e5bd90b19ea88088a16573b9478a8b0112f92b8b43ed11f04b2888faf8d52ce458ced18abd11941154d1a2ccf5bc3",
					"0x9169d2908eb57298fed87820a87e7351f16a8634c4761a8bd6c4b5d584444d4bfea86c26ef58e26c64aeb4af1f6d4f22",
					"0xa2f0cfcebd69c91c20584add9e7c5d0ebef7aa707da66ce05c4655ca9e4a61b85894c53b106a40634952de1a24b7f2bd",
					"0xac53f041ca7c28a28e9b8af3b267e73104a2b0cd32380fd040d5b43a75233e5b289e05f549847f4168df0f1f63d1ba63",
					"0xb0c1c176ca0c776b0ce1c873b7ce11326fc16dd3efa0c17ad8bc32b090b9c10121147ad6d3e569dac90437c3ef25b15b",
					"0x8c9222d1ce04f579680d42c23d77361340d80ccb39b62ee0da8579e43c6ebae42bdded7176fc3d44fb47f615596653de",
					"0xaaacb4f33af8916c2b25bcdb971d1c4058f3c0990ae7992731ded19df853f5fd1d3c578fb17ffca873c76c5c4e3df560",
					"0x88dc697a2dd7e2bda728af091235a0e3aa02a4a68349fa1c48212bc0c198315815c737424f4f34a7e4fb7dbfce587842",
					"0xb9d937f7433742f80f32e1a871ddd994d5f09def222d724479fe1b69fb658c8958b2cf95a06df3b6770453f94b39efc7",
					"0xa422946ce08567f785ba86136f89c17f74c021cbbbf8b2680b1a99fd56218144af748dfd78d5175f7c1c3756fb32e411",
					"0xb9ea4b957ab69aa34827b3cfc0bb18d713ecdc6ea697e83d525188a11bbf1f1f5c9be894c9c8106be0d9cefa08ccef49",
					"0xa4a25f790f5008b132a00473a1608a87cadafb441abda9c29298e1c2b110593c38a6b7d2df8ebab66b024a58f35845e9",
					"0x986a1e23273adc33a497afe4376e3995b88f02cd997ee77950d110fff2913bfa5631aaa9c6da4794f7127c0e34b37b21",
					"0xa6a5e4e9bac55c50b89dd4209c7620dd4d8833291cf365bbffe8f6d291cf2f2e1316ab2de433a49ec6ae2071cce9b925",
					"0x95c1db5f1a01d5e313c642833c1a11ba76a9003f7f1e6a0bdcb84c12a72d0c97ed4b9929706681b45a2192dd6cb1e44a",
					"0x989e59604cf3248c1a9616080d66a01829454404057fbd3ae48ca26ee64e9eeef0722ce3d88398d2d57b07f60235a29e",
					"0x8b5e56b5279e7fa4ef1b5f074dcd7ced834a9e95db662015a0828768b4373b1335c8469774b844df049d287d82632f4f",
					"0x987b1d9519f51f071d9cb6e097836fd3b7dbaea391ab75de8ac240791ebeda90380bbb94f31517d56495c36cf5a41824",
					"0xb6225eb448b76973934f7907dccd672bef955187d6850c6363117b37bc75b080a424d3d2dd5c9a0fb3721739aa28ba8b",
					"0xa60ad06fb780b9c20c7f7aa8b32cc93873c722da0ae38933b3aebbeda0dd12bbc04311b0854df24c6798bf7cdc2f9210",
					"0xb0af6cf2c9c1688e8aad517bf177e65bfdfce6e7711473e84ba43e3f6e7fc0a067c9e95aa831a545c69975a08e1ad6c1",
					"0x8940392de1fddf5c9e1d05f6ac6f64af9967db23c51511cd4729e10ea60f757c30cf6f03426920fa03660cfc2e131d15",
					"0x93bf23a587f11f9eca329a12ef51296b8a9848af8c0fe61201524b14cb85b0c6fbd3e427501cdfa3b28719bd1ed96fff",
					"0xa0fae701c39985b45b9b4f01e7d86612dcd16a33f994ca9059e79d066195a36712c414fdb1516a462bb9bf21bfb5eb93",
					"0xa7c61a3b0479b71b41921afed36f61b0228d758113b7096ac774d219aa8d1842dc7bd37d75b1401f5e7eae34c2020a3b",
					"0x94e167f5f51560d29c6a965636564cc13e56076fd57cbda2225160356dcc2f0e0504f607f83fdccb61d4de9ea26db00b",
					"0xb6961c78c0f1666726acb7b2aff0be2812241ba3d138b4063de5ba56862e6d43672d250bf4f470cb07b4e5a977ab66b1",
					"0x93a9028f6473982fa92b1e0df1bc6a165d273779b391232d886f6b2276b2a44c9f31c816dcae6e5ac877a6074df6060d",
					"0xb32e2ef9dc181f03136c2dc2b83730726dbb886cafcab021024ef0fa296e86cc8542cfb1873286cece8f5fa18cfdb631",
					"0x891178aa5d95a1030cfd08f8d7d036b4512ece59f12e532858e5f8ce5df5a73cf554d4d3be8f0c369aa2da783bf55b51",
					"0x85c6ddbea7accee76157a1d93d82b461c7e0fac5e8b5c2c480923592684f7f4c627c30003b4aa382bd665a4e3a5d6a8b",
					"0x8070515c07dddf2d3a27323f69b3d5e8731ae41c8392f6105586ac574194026dd435a956f8ea2b37f391337cf1977fd7",
					"0x8e3e46f9a3a61aa740e3bf075124f1d20804fab88d9f91f5840b8234bc1a31bea924a641e69d17d53f7d7ba57c03cca2",
					"0xa64644513fe9fff133f3a8be27e93aaa94ba9dedace5c7d68dde29125632438d887de9e6a41e546319b12ec46381f46c",
					"0x8ed9d4e6aba86eca4abd2f4a69af8da6c9e8021e0f9f692fb57c366358054c08f4f6d49328cac6a44b4017ac5011a111",
					"0x8cb918184ed0ea8ca09356c5c5a58dab29984ad02b6eaeafa76713b5ed75876c0a912efd0b3c20b43f55162320525896",
					"0xa1691a8b707011cf764959ec29afe6cf10c0075d37d91793260582af728fa5dd1ce6816a7690acd5f850241ac4859d37",
					"0xac563c181042ac258617e70eed2e951fcbcea6594df6a73f9587459252c2073a01c0f70b9e76ea45581fbc0300058f46",
					"0x837d7e7407cbf299c7e7b628ed1326c6889a687ba785e2279895e36c63d16be2d5442cce7c8e31596f0364f906902a64",
					"0xa6945150c52a3f8e30375c24d1ca57e3414c9c09cdc1c4527f147d58b21d400fe6554908d0871e0a8898f9f8a490dbd3",
					"0xb94e2771fe07a7ebe62206a0bf2b35c4acf06bac4294d4bb2584cbbac2888420fe382c16730c2ee7d0dbc1d8534335d0",
					"0x83638412af0d2a1f9523f7a7b294eae8666615ce4dc5b83c9c712379c884082148d269c178225532aa3f040983a5d966",
					"0xb8ebe4df1e4ffa4dda3da3279d00fc0d0f4e898299a87323b96262da295c983374ce26aed3ffc1ecc9b9b75fa8e13981",
					"0x8c27369bb7a89ab075e7bf111b2899c5cd1a1799cf60d0f44dd4147dcf5ed33cafe5a17cfe2e24952bee8c5c08c77054",
					"0x979857b6432826c83a3048cec7a38752484f12cc50e1cdb2d23ad4306164e5d9e3bf765aff2f3d2ac53b3be479c64498",
					"0x9559737104cb6608b7740f7c1cf692eef4af79918b9d4a29d4244a42e0ccccb73948f334c54759b205d96fa6848c209d",
					"0x94418bbce01e8a052ebe00073877cb6f428d49ae8f202573104843db206d1162d9d02257355c0a33427b03a1a9ce5714",
					"0x8c038297767a0f5862f35ea67c571c600f112398aca2f0f37ffaf39d8574ea874e49904e5642d8b02c61609e5acd083f",
					"0x90bd5f97e38f3f740cd1cd2863428681c9b612e2466fe0b2bd649b6c16cb92416bc78cd4d4459d7085a3babfef419d31",
					"0xb918e37b4a4795f769ebad5d29bec0ece49801f2c669d77331667b9840813434215cac290d1509c1cf2397a448eab85c",
					"0xaee9994a1881e30bae2975bccda70b60cb00716e47e96b4f948e58a1099adad1aaae1de0538d8d7bc7a09ae759c28ad9",
					"0x917b8f9acd2a040d112e5818e42473eb935106cc753f755f9b8de77c7b2340d89af3f9a4478b938490df158e0f0e4f11",
					"0xb8a53750855585a6b5aaf957da03991ccaa837c6f6900a832ecf31a1824799898bd89dace661bc78af245af09c501595",
					"0x8df2d5157981dcb7d14b0027998ae971eb8c1f7a15fbe16636c835cfb1b24aae08ea9dfb7504d78db1817497275466fa",
					"0xb3b1a129de0fb8b2a3701dbdb3a15572ad88f5dbc867999afa71dc48c48300961388fb6acef5e517a7cc67cdcf82f9c6",
					"0xb683aef273957a70fe1050290dc719e21c0d8d31f5c78a395a624060f9c1013a18c31aafed3e00bca1d90f857eae7dc3",
					"0x87da5a7b391e57a024588beb941744328f7a04769af9bb19822c9377232749ba9eb7788f47b2365b513039006b64f521",
					"0x86f7bb504648a236cf0c2edb6f59a79e07c0ed236b88dc28a56bf2d190f68246e7adf8f8c4ac190514929e957ecc9005",
					"0x860ac8bb64536318f1e2e88233d64e428ee9d59d1173b2ad174b71a6b460d23c91a6355d387da9138ffbcf34b32037c0",
					"0x93fc4e6abcc5475eeb1d636fe0716770d39aae13128e9b6f4dc216513a83ba595e0bfff5d04db68604a1080e20717c7c",
					"0xab0c556b7b551abea632442c44d5f41a55529dceef3ec45167cd5b4bd57d690b238c8204f791d432e44a4ec2a600cedc",
					"0xb913f4bfffa022f198cf535a0a63ca19906a81dd53967ff1a63006a438468cb082bb9a4b903d70b80d7b8cddff0e8d28",
					"0x84cc6e28cacd206a87f0b8fc5cab60358d32ef7ed013eadf5888a7f0bb0cd692fa912fafc1b54bfa3ccbaaf1498e91fb",
					"0x94f2940e598a7c7647dabfcb864c0eaeefce9dbfb93517385eab20b3923e90dd86c6140badb8008dcc26123002d94e81",
					"0x83f0370ccbee907f533428a5fad89b876a209c63833d1f253c9654db504e09a7f6e23312d063a0f64edba1c9b84e7fd6",
					"0xaecb5573c4df76502ad396bffa33cbb8efd61ea1fdddc4595ae8aefff368b6596a3727b35a43b78e0d11db034cfbef29",
					"0x8539ab5c85e2d01c5fd13b8952f7bcfeabc712cb246e51256a189165a3964b2984e2272b6abaa175da42c3eb9f45cf07",
					"0xa1329b4d04e266c7ab8fd4e67e10040fe20f15864cbca8c4b6cf5e5e85a639518f62195654c5cd18de57921c57adf4db",
					"0x8a9391af02a40b5f53cbb51ec1b9b2595e8fde8180b2f6d787ba948c1ee7b18a3ec0bbecdf36d8538fba3ba40c6f18d1",
					"0xac64690a165f09d2ec3dbcf35c9037cf3c00a45287173eaf0ecd7c89011719e6b9a342cc6345be31772222a16ef02179",
					"0x85ccd43ebaaa9aabfb4cba250d4639a6a5339cd5b5da002404e42164c5a053073c5d4e5ab52e31b4135053d150400282",
					"0xa03bcd8849efa0e03c8b110c8da260ad7a39093439617db4296c3d75eb709093da45c2403ebd43ae7756d132c28c4606",
					"0xb616743e11c0ee5333c491cb239c538c2fcdb436154dfc84427b8e4d3675e290c5b99bfe04e132ef85ed634b38106152",
					"0x8230a6a61c7fdfb5cb96eb9e290b31274cfe47d98ae0b82d0aeee680ea506b1ac5350e4b34f5eb8194d49a8666e5c540",
					"0x9716074db84c8e94a046e246b6b9f0f5c5abc3d239b7364bdd97b4d285e65dde9ab159b2aa0f7823fb518b12d94ec491",
					"0x880e03fd9d0f726bd0b78978fe135f3af72a65e9623cf050a881bed95515b9ba9e220f072f84778b0534f1fcc735e80d",
					"0xb66f4ce246f5ea336d35890467c5b6c1b5307540fb657543e562a96edd474c439b477eb7c1a1123b972099d5847a2c06",
					"0x8e4b3ec122c0fffb565c4d49932295ba445c15eee3e19a7c6ee5ecf0f345adab74a066d37f0fe7a2482c63eb44068f68",
					"0x954d5b24190058b57f5c305f3e80d97690723a7b7e3a2852323aed8c7da591565304bf913e737db82fbd962ed4e152b5",
					"0xb1701acd767b0c27270c92e9674475a35049aa5008036b04a6bfb36586bcfd8c166c6d17cc92db468fee35e1eae6fe0b",
					"0xa81374387052fd4121be7e4c2468a6798d59f50492b4de246d5ff286dece04bb1d1dbd3d25d3630a1caace64be8e7a82",
					"0xb66d139e18271b3e21e3adffcb0cd2a4d71c073516b4a6c2865246bd7108d61eaa683a809113db3df427c018fd774472",
					"0x96e7fadeee2db315c544ff41ee2340ab289d669b4fd69e7cfe11185e41e273ecb7f83924cf22cc712bd2621e758a5563",
					"0x8be4aa4fc94a12f5e71cd4fba81a2a8fc38be2be735c1d8e56c43843a9fccca276b5bf78cf558bc8af4a81f82d5de241",
					"0x99af980d3e434dbb3a971f9decbc78cd663330e187a5a64165f7aa10e7e77eb590add1418dd76e463df08cc678b3b970",
					"0xac550524044f7d6914e5fe1669eccd15e1480c309d03f607e724831bf77091c5f7340e92345146c804792de80e7c24d6",
					"0xa9cbbfc8e6a2fb6a1f292db60657fa581dc9252b8c03f3c18b386b13d3cdc01f88702968d09cd47e635c38d41544d201",
					"0x8bead6b86fde82c7ed3f7bc2206e2c6a15f1b69b79f27006511d7311d41f070345533b1239ea3d5f9dd971d2eb52b618",
					"0xb5b09ae6aee25a83eb66ae95cf40dfb572cccc5ef436498c0de0e5e82fc0b431a2b5a320e4daff0ed59e600c375daf29",
					"0xb464b18b620e470085a2ef9e4ebe00db15a10913607e8988e76bdb8028cfb0135b6e3b88a6844e9625166cfb3b2fce2c",
					"0xa1f58707374de87c4818d4181b9910a79bf061a47b189c14f96c311c35010396f128ebdce685c42124535efd44f58f03",
					"0xa13ab98f405ad19a77393c70619fa7ec530e41909509752b6b13ec81a402b2df9f5891491959696347fb6e67323cf211",
					"0xb7ed5c9da64315df5a724b959712b68e541a7bd7befc3dd6d055b0684632781eb061f760856c7ca0744ce57d06e497c8",
					"0xb9de14675b7b7025959af1fdc64a68614615e410a5c220cd7fa9acadf1b8cede12b913d576f73c7a746fd7dd3bf92330",
					"0x91be0374d30cbf56e310fe7bf190f5ffa86e27df56e470164e08467d69f71e4931555df77e170b577fb6ab79f39876b9",
					"0xa5d42d4cd07c013bcf1cfd5e21886f481d43ee4275fdb772696f6fc047908394455dfb513c63f5f1cd67fb2970835a84",
					"0x8504f58a6786b53004573a64e454bf0dbf1400ce0edd5427beb5aa84b9a300caee78a595253b359bb96f9fb9c914fb0c",
					"0xaa0b4edb0977b748900cc74551a2bfc47ff520c9358f5632cd6d5924f9f966b3ea543606e8873545892159dd0f2f6bf5",
					"0x991e50fdef4c55b42de1bf7a9eb8460c0c368a3d7a74ef632ec68a01209a2c0b3c5e3bdd757503361c13917da7a9a4ba",
					"0x8dcd06c0c6b8ffae1ea992d0ada16b09eb72184901d8550793147824601ea231905aac36b88f3fcdc720fcab0029b2fc",
					"0xa01ced80dd25cc3bddd6c9b00cf11d4c08901687be0a14d3084371d70e30e58eadc031a7734048d76402e22e00fb0b75",
					"0x8d53077330db262203895ee062a0c8676ffa62b8284bf8421a04c1573662ed9763707167c710eed5aa5cafbbd9b5faba",
					"0xa4f4f3395341b573b46a475984660fcfe28d2a699027022a197572c53ee5b0a797332e9e1099dbf5abe5e914fa8dac8e",
					"0xb3485ddc9587eba620edc9e3326264facb9b259d888bf8e12dcb61f068fc54f20073de14894c010a91f146aa06ff18ed",
					"0x927d7796d1209bc3a48731342995798e79c13ba80f6ce3cd5da4c86b521b9e419d4e6c3effe2a0336b60ff0a82dea546",
					"0xae4c748495f57511fcf64286bc2ecbb752bc99a9b4e78c21e312610075765c52f5b1016bf49fe93f36558b96d0182796",
					"0xb2e00a7bb7b02ae1b6ca28a095bd35b6720584ebf10e4f65a712548698701665ff8854156a9d7b839af50f06a023af9a",
					"0xb8ec49d1e759f8910c30ee724a68e9354b5b2e072b77adfea9d06e5592cbc54dd953d5de869eb4fd0be22db0dffe4761",
					"0xb6609b26b177699b28e786de3c92ae6ea32bf397a83eeb2030cc1fcc0e88f9ce08ff739dca1d62a4c525d3d78a666cf3",
					"0xa85e4ce57db6e4da20a1dde939394585d2571ba56ace6c0054fa76db22a8562c7c2c93554223b468e8471ed5ffbe23b1",
					"0xa99ef705c2244baa7b624071cb6477a369cd1ff0cf90d42047952d4e7e558dc2e5459cbccdbaac66767f8a73f75d698e",
					"0xa3e6c40e38ab2a67448a753eaf2bbd59144aca22dbc849e2634aef773a62e0706dfde2e60f7edb23e23438a81e932f82",
					"0xa1fbfde089094a669f265696de3c84a629507376ad380f6da346bf822503eb0d9f08ea8b1a5bfad3f1632a008cfc057f",
					"0x8d3eaea4e186d6d1db734fea85c2688900f05ceafcb984df0301e501899a1d2ec4b5572ea60ae00e1af38dfa88389fa8",
					"0x9213193e86817ec6e34e49a1f05b171be068095db9910de62900e3d649f505d8ae04aa9f899edabc4f71344c80c3addf",
					"0xb73dbdfa6e5d5575857b3c55c21ab67b190a904bc2c0042a4a4fad25edf322f88988d858156d3838900203ed027b0323",
					"0x91a0df0a7d06bd28290765a0c83fa8cb337bc048079de6ec667f5c3075c5dce586f8aa9d4bb8a52373ff8789c04da387",
					"0x9822ceffd20247a06dc7f003fd7ff45f1027eb7ad725843033b6d1a2ca44017efa302d32a69803bdf8282b23e6ed31e8",
					"0x96f3ed3e2ea4854e50c25b37fe72a67a6a9b463ec18e06b74c8c16f4eae455776937059fb5063bcdeaf30d47fed5fcdb",
					"0x807bab983ff842e24e5846c68a78c837dd0bf8c177f606e9c83ba6a33f835e9f12a06036b9d5df958a5d4af6ff444548",
					"0xa092d998b5cd05739e0cb7e6b86935ff979582e8c22347fdd9d4b314872c08d5158a2f44c7cedddb4aa24ab260945556",
					"0xa41f332bfe46b3b6563a36c2a22e32c03a43946aef43d71b263636979304462da195700d73071106ab589a71b72f2ca9",
					"0xb1f9e3742884bc3957fc02823da8c161595a9432b8e79d3cb5581e9439a7aecab6ae8797a48876350db410fccfcaff56",
					"0xb9fa7372cbdf6e72d1c8ac9e6edadc5f482616cd13c0fe6cf61a3c2b121635d8717c278f444d644e420a662af4c9e929",
					"0x832de1a24fb910a9437e84f2f330421ef0cf6026a088f5837b8733c2f89733257dd274764f65f17398ce22ca995d7ca5",
					"0x8d579d1eab6e29fadbc4ab9f3c1e53a2e569d215cec23360409aae41a22a41c91a72058e785a9270e32ec0381ec4c59c",
					"0x82190dbca9b99f4dfcfd415d7682cfb3d652f24710e498e7a4040d3537cb6f8ed5b020774d4ea28fce43a00d154b9300",
					"0x9707e38316db3dedce657c6962c0820f1d427e0a2823d56a1cfbe646d1781a4f0536bbc3dc04b2bd3388a010d04fd0e3",
					"0xb8f2c838fb27cbd539ed8d34178fbec2f0359b628ae91c2a6b6d39dde81d88d9f45f602f6ccf1014ed4bb0ac17a12064",
					"0x80bc2b6ee0b5857bf1e49fb66b68c3226aca2529538c3380e3a57b51c4cb3f1425115b82120b6e8b423c3cc98312f1bc",
					"0xb20d9e8cc596dfb55ece793297cb51b19003b817af8e55753ccee037d8e84f5188bb087c151695e5718503eeac9f49b0",
					"0x9574444420c14c1b5a1bed6b56d9e65511afb389ec183e77e6ed7c3efd919696f96c11ea2f0829474478fa831f79593d",
					"0x9068ca82f1ed28d453ce939c36cd767fca528f20efea5273484cc72357b4fac4e2b18315e5f4ef490a26dd2e9c4aea05",
					"0x9147e3fd18fb38546ca1ef43ad2c6a5d6119113be3ca3afdc2480f57b436eece3e5fe9764cb1dba8bca402a367f43f80",
					"0xaa8b45d16a010bd00a59b8883b2cbb566e341de4999a6f41ea296eb55b8879e159f7d83457fb4870907e1eec0a636a10",
					"0xa49cbb03aaefed6050c145d08590b7ba54a3e955ada3b61f8561061f8fe295cfab0d42fba9a95d36ca8364e462cdedd1",
					"0xab488ceb5d2499233637843d8903238470166cdd1449bc9a06c3032d0ec19b5a06ae54da12bd3fd41dad8ad6208d6e76",
					"0xb5ef364c19495e32b389fc00923a1f8fffd0847eef44016e66f9a552e6accbfe4c56f05874c6c0818b159152be51b719",
					"0xb9072438bdb76895247d0cbee384a3e631f7735e7994c709fe8cd5223d0384d1feb8925a0e977901f773bbc5a366e048",
					"0xa9bc18688a407f27df306363f99b8fa11b939af044571ab7accbb1cc68bf038bd43044ead2f2c602f4e367314ad7f4e9",
					"0x83e6c807250c5c2d05564eb5bfa0ce940b4945b210888b240b0dd6238e97213a2beb93b1305d3f28965c0ae6b27308c1",
					"0xa9436d1080c7df01db69d0aee0762c6c988d82b28be0651ad5c860a459a249ef1995b5d248b352b38f7cc46382394ccd",
					"0xaf80c559e1f6b9d2c78e7fd073b1049109ee67056fe43638ec88b498ccd6895bfb8510d2015ab57c8dccc8f8916c9cef",
					"0x80c18d22d2a7523aa10bb82271d74372595580af3de3150df205b2161345f17520938e36268a9cdcf3629282ba24a1d5",
					"0x8a7ae977578507b3df8c892e9aaa0c6daea918d7ad56d6c370c20f2129ca364f17e4dd0075cb5e3728ef48d500a9a54c",
					"0xb4298d76576f1232718f4203d2027e6f92cdeb8f6d619426f17b392c65c974319607dc7927541ff8b1ccf42242546b1d",
					"0xb66de001b4aafb30f0ad9a1f3febda1e2ff218db5ca21df229d29cbd3ed2a439a60253064521f60d73b8e5c1d81caa7b",
					"0x9357a18f1995eeaa86ef3fa1e32d483d7b3492d24cbe3d134ee3b45246465dd5a9d0d4d54af4d5a1e1eb23c5590ba920",
					"0xa671c2c515cbed6091195713343f05f2425e493ccbf31e6c01b5f0b26cec66b243479c8c996beeb7d18a95ee071c538f",
					"0x8079cd266f8444ef3a70cd1431fba6b2e1b85d7064ba869d08474233da73d717c003e51b19f0ad2e0ff8743b75c752ee",
					"0xa1ef05d65bf43b51df5e072934522838e831ba28c5b49ac9f7c9cb85e17d9a2947002642ff712a56a5f9f9a760d794c9",
					"0x91886f2bca105c6b706653e6c3a1e5802cc77134af5c949be1be2937bb2f2dcc01abbb0bdec05063736b6518023dc215",
					"0xa5d0734e3be02e39849244ccd0be6778f14cebec27bd9e14ce2ca092e04863b9587556b06201dfe5c17d742a615bed4d",
					"0xa494f6f717e215bd3d7d9d1b88ef53a04492f0e74690809436f4f8abd102fc98d48ba496302a4c8938c640831a4c4e1f",
					"0xb8ad39db2f201ed841068db62c09ff9689658d9748c83bd99ff51e328b58048c05ba606918266797e6dcaa86d233d5fa",
					"0xa390c3adab683f3035e850023d55ae4de0c1d1921c19a866448dea3e7b2dd3a121fa1e98e9536b2a71ce6b39cf72ca72",
					"0x9956085abc89f5b1c2f84267c17439a29bb59e3e2c3af1778528a332e7a31581bb4e921d83ea329489ffc19a2158f15e",
					"0x85322d13e956606c6543900a9cb40d696305c40fa335927230c156c8a05fecd016822503b30e226fe8634a47f3709f33",
					"0xb7e8633962fdf2fb1e0c3d15005a337be710ecec43e10a7b483f4f9a2d96ab891754e295e32a1afd43f782b091bec57d",
					"0x95c5a309ccb2d21e13aad6ab271542850a9d1eea9e862c1b483c5bd3ad54f3f60c64aa964eeaeafba45fbfafc0cef151",
					"0x8d045bb81b85eeb7fb85331420befc5b1890d897035f94d475c586feb2427bccd159f3f45cd4cbd823fd7a18f333bdfe",
					"0x931b4e17344b1d47883016d97b053254c9c19161950f7a957a28de7b564736212211cbc42f6b38d28a049541276edb16",
					"0xad16d39bb3148d9bbb1171e7563386b275367148342017bf7e8b86d9f6803706a8c7646bc753420f25a525e217d61cf3",
					"0x848b231e23dec203734f45c2c0a4b9f7cbc961109c320da8653e069fb2ed61243d8b07021c04fe0fe8b98d5d5e49911a",
					"0xb14e4452fb8e8dab053d49dbc151ce00c11e28d0a7053a576ea6dfdeb893e0068ad8318c88e5d04eb5149f16313e9d27",
					"0x8a35ba072b808cc53fef3dc9ccc23f6ea03ec663e7b479494077762d9cb9aa0a34106cd1f84102db5d9dd9eabba58738",
					"0x865423124371cd6385fc75e7e687cf072191f10c6a53b304a48244fc543201a86cb8a128b782408d0dda36101cae4e82",
					"0xb49621a87891d8d183f2f6b691b4cdd0c113d1c110943551f0006f77cd86661f43c3e56262d47186c8a1899b62d872fe",
					"0xa43bc1aca09ef99680a2ad65b110c5dcbead6bde4894e0d2c919e7d0044b458ac96a9391150bceeaa02ccf41ce064cb1",
					"0x975ae297544be16f64aa9b32fa6294e3a4fe7eee95ade53c0e622dbfc6afd5c601978bb9ec4358972effd69ab8715f9a",
					"0x94ba63473ce6ed093b4202334aeefeeac805d939b12b92876eb566bfa8bf882a040624f0e4c68421a956536dd5334c6a",
					"0x88f8f08c26f7f637c8a1f30d1cc2ec10f74fdb2ecadd55cac1aa9542ea430a6ecc95ccf5a17b96f6ca01fecc02df22fc",
					"0xb978247f35234e147d84497929edf3509a0b16aeca6ebbb4d6aa4d22e28562b7982e9ffecb865e11d41da26113950bb2",
					"0x8cb5d5337e1f605d1032fddf9327c38717596c2409cbf882a82aa1461d8cf82eeee00f961223a473fdd1ce6a55b3916f",
					"0xa672d36d206e866c2baabdbfe8362a7dc79826e52f2fd4b3423e6f5deda279cb8ee81a1985cb20b6aa5d350d773b5d1f",
					"0xb38725aaabfa2440114607b8697823fde9726fe0b136bf1ab66caed3e155055babc2c8fda647d69f9946eae40b2f970b",
					"0x8c7d385080263f20f5a7ec9f254d6cacf4439bb6776f7fc87d735f455f27cf5139dc0cc907f56a840355238f5a20682c",
					"0xad1669966d3014128ac24a6b68ec11f5ebc89015e909818d9d1e997d7c50eee4a40558ac347b217e055ee199c0e54554",
					"0xb985c07f2fc573b8ee310cfefe16433c5623038eec6fab8aa4d40589f8c0069333fa3da7594d83f2abf1c3b3a121f88a",
					"0x8ffef8fcc0d1de0e1a6bb983e611aa339db9e7e9483d390230d84172b16410e56ebd25c7f0fe9cd961a66ebe8d72c246",
					"0x94561969de78cc07f22c6efeb762680d5414b2db6debeda4a10f8e1e4938faa87e92ad37af42fb674411d76eec25920a",
					"0xa36c663cdba05a23bc6380f19d631564d266a40b0f437945985a9b056e5ef2ea5fa6850d801bbc131d2faafeaef96e61",
					"0xa1c724b8d4d6334883b0efcdb344822f2454e8bfe5203e0046f4b3d144d97f2099749b3059646fe6fcc4628b877ebb61",
					"0x8d3d1735424d174075207bfb8524038858268f3a4460ca749114d751c93bc188ddd6c925dc32614c621c7dabce11140c",
					"0xb58e707bb38b302b88d3cac85a72eef1b021af52ebb4cd2dd68df8511b1c50788ef0e9b9c5b20ff09291f9f0e6a31a4c",
					"0x876e3d6892fea2fa31f772dc7db02c16a4a103703a8bd564300a36cdd90d8cd5c11221f9523538cca65d2a79b79b0b42",
					"0x839bae4b693a2838b9ed626e91f786d071dd9e00ae6b45f6e463ede6a481415c2e690f2fd31ca0f9e3ab204d7aa3b309",
					"0x838238038cdccb7b2603fd8903b4032d1210fccc2abf0bdb60100cb28636fe5a09bad9c56dad41deb59f50c95eb63d88",
					"0xb44d073315b80b3e852743f45b569e94f794a63ab88be361f69257781b34a55103eb863fee27752ca6d0fd297a5f1bf7",
					"0xaba9e874cbc8a8d9d037860f8ab7875cc349a39eae74d27fa42e49f5678042a627f6cc560d824086a75ad6b4f7e75d9a",
					"0xb04519bb6630b83af542860c4415188cee207e4953ee552872cfc838deda4a48362c208527e48a3b19618e4d42a42963",
					"0xb3e8e86c3b1a51e64740218193485c3411b6627cdaf7e8d71d51758bc86642ab2cfb65bada4b70881e8aa24a365850bf",
					"0xb7ebb569aed68d454c0d6192e2213eb0152e5630c6ad87321d942d75125bc368a47aa6f3c5da4fc074412489d0d715a5",
					"0x8c41bc4cf25a98d48c8c8df07a1919050ef4f88db679a215aa9509ac43f05bc6e94b1da62b6d172816e3705499766a1b",
					"0xaeeb2dd61295f308a4a14a798d4da45fbfe27767d0f511259b82d263c84d65b676990502af9d299fff17160049534cf7",
					"0xa28eb494e94b619da70824c8ec259cbcb5593c8037aa1c08ab7b28f8ab6e7ccb3bfe5cdb94f72e2e07e19eb8ed007f0e",
					"0x841f1a3b1ba2bf2371bc78caa773676300b616e3082e4b32fdadd07eada2bd47147d11f6ff2cbc0d569acfd1d5503835",
					"0xb79021657f2ed061ee963bae18f465356b0da26d2592e0fcfd36a0552aa88f8b7d9ca3ead4a0fe29b9d619dc6324e46d",
					"0xb2e32fc0a5f9b4ea4855b07b57b92a234ff4db5b73473f9f8deed039f34db45364e02bed9b21fd02795dbb042c370fa3",
					"0x84ab252d8959429347df96e5b70b1af1ccafd7500de1bfdba8c7e2a87e67331716d0182fe12d6aceff279b4bbd22cebd",
					"0x801f9278a6da23f7a924ec88e926c4cff0752b353e3d2247db239811d0585b4b13969dedfa6148cf234b3d17e5486068",
					"0x93860b6bff17d4e78ee6d56884e989c1e136ac8691e4f7449c6f3589a643077311b1e4baae9c0f401121d65c3621b85a",
					"0x8f70d54ed786c000ba525a706f61cb6cfcea034161ca3c3049e623babec567803b5d176c8572c47baa55e7276d9524d4",
					"0x9211ae7771b584d52539a5f09d98b3a1ba3611815f0914f31693e52a03bd04e2a14f472b4e8d9a85ff0036e34a3422cd",
					"0xa14bdf3a2463a27e9ef474f6e69c39c8fb912fd9760b183bd2a1940181ad3bd4b877bb9d3900f0358506b79f487d5fd7",
					"0xa5bbc658e5df21d667108682f2dd1555df422053cad7a63182f5127960704d0af2c5667c5315ded2bb4498bd38776344",
					"0x90d0b389877525a17c20e43e0ab491893bd469f4c3c38725e6697004777abd1ebe19d9536247136eb790328464cd80ad",
					"0xb8e448eba82cefe9216ac767c7e5460ccbf00778aaf5969e76de933142c9f49db14e19214ad30428f16baac4cc92f9df",
					"0x809c4687281d592c1235bfe3369cde1198855aaa2bbe31b46d3b4318b30e3321040f74f7069c955e14d4f08e4ede45fb",
					"0x99a4bdbc559235797416fbae1fffc51c2c1c4fa5f89ae34ec2dafb003f849ae239f494e827d2bf7bf3aadd8cd395764a",
					"0xb0b7c8ed603bcf1a96a9f79926039bfae42fd8eaec8d721b0352285041c732abd411b14ee373ce60dab2ca458ef2beba",
					"0xb25b42bf72bb3911ab9355aaa75972836877a3c1360e1ab243b9d95feeb7056c5edb89fc25d5eaaf5e49a0cf5f790f79",
					"0x9749ffce13b359ab07a0aec5b66c302d6322f83e1696acfea333c823bbb99a8d9e81dfdad6ea54fa1bd7f8707173e399",
					"0x947c6035657dd83835dc878c79d40a6b153841794a1b10d05fab5602ba126524cd72a4f0f7145eb46fa9826392b643ac",
					"0xa7440dbfeae14c032820b9879d978eeb689bc4c4dc9911b5a6176852da231a844a513e3c970e579be24ad9bcc9c5ca18",
					"0xaf10aa939974996115d00a52c969a3e2354fb14b019a6efa516767d7a1f22333307fda9aacb45872583f75366e2afaf8",
					"0x8e02cba89e3e9db26cbbde6c0db047c7ffae8c5cea4c6468795200a5a2204425b79e77e6801823b24a4bbd0aeac17755",
					"0x81940a6e43bae391e651e2114ea90d5dec4e7969c902da578b0a89e5c87bc4095a445ac0bf44c32bb450ed06a0e7dbcc",
					"0xb86b677c9bfdb27b98d1745c2fd0358570e0ef0379351af82538a65b20deea54984a3afcb27611c0a8e3b6929684c568",
					"0xac13360fe464e6e992c915f3997bce038985096693078a96c7990b74f51d4ff83646f18091c3ecbe87f2b02f474b34fa",
					"0x8f269bb2852beb00fce8d892fcc568d56b25e7cecd018420dc7ce9c1a56e274a8ae562a116e93a8afccec58d743ed2b9",
					"0x894f4d654b5dbe65eb5b6b75a2e1ab06c887586c46cf5ea3fd4a9e056166b6c67fd7855f7abe17b0f46ee5de33bea5a2",
					"0xb6324c14062ce00f03641199d2a9068e5a5968a8b1bbdb0b7d465d48d7a556a50f2994325c9ca61c0c55899f9cd0709b",
					"0x88d9cd0aa47dab39b7eced9f4552694f08f97376bb419cb6c10f7879f14b466e35f1df90947cc91730cb88ac4f541004",
					"0x9582273145cae4e8830eaac6514a85727f9e506fd1483f15a74ec827d4ec3a0009e5a327bc11ffeca884c06548a89332",
					"0x981c7ff734bd5cd98299a25f5e0ea7d65c8e5e8d9268e05c4a0176bc81eb59d07a881cf1fed57037b794dc80890c058e",
					"0x8b14b6012b584df53383e14149ba038f9fefcc506d1bab2cd413decd03f3e6050ada547239d8808c857b85dcd845f4d6",
					"0x8a04e7c33624979ad410d24bc1ef750984421dd6c8862bde67b45cd3092b0bffba611624e75f4e62aa5ceb2c329568c5",
					"0x8bb0dbbacc0db4faa9fb7f481bd17ca7bd97fb52668262e0a1fe6930eef9ee30eca37d355ef4f8b6e9e0f5e0eb5980c3",
					"0xa0e541897b615f1e2ba7579ba2c3fdd3831a124068e048c044cfab61e4fcaa8563d0a229eb1d342d8066f1df9893057d",
					"0x8dc79868a2c4c91f918cafb1f1ea1f03c8c24975aa824ffc990d5bdd33f232b687ed5c7ef393e30ae16e6644c9c3440b",
					"0xb0e7099f4402ee25a9598de5c6ebd0129bfe3a495a6119aaacf5a76419beefb6b877dea8613b6b41b03bdcc57828b050",
					"0x90354f32520ca195d124308ea7e5bb1b0b67a548b9904f66846417c57b36ecf0208aee91749db2cc51c381da752ccd2b",
					"0x924a50af6f7f66d530bda640a4b7d2bb40d093292ae51e220c59ffb16604bb16abe08f8aa4139cb7fe58bb9f1f8aba9f",
					"0x85978a2954c239f062d7f02a70e50815d873d1d08c0ef9810dcf38b12d2f4430a2c90070d31bda1465b6176a6751ab95",
					"0xb6c6d84db434f027493fe94342e275637f995da6dd2c828067c1d28c7d643ca825478e37c3fe0d06f1a320be665325b3",
					"0xa4cea2c2ac6f8b636c2e2da059dcc6cc6e2a3364fccc3c15d9292512aeffb094bda6976d38bb651f59f3c0123ca93ffa",
					"0x91cdcfc2276aea868724e4aa3093cd5b33b44f4115b1f6d66941165585cfec63208a1a610ca702852d69378afacecab9",
					"0xa94ab3dda8dc2df72193c1bf535b8b0869fb58f2549743c373f87acb0d5e7267361bcc46f837b2b34e2067872a53c5f5",
					"0x9774ad6ae35c17997c8edac5e2233921dfd4f7a19a50c98a7300dc07a572a6d17bc179adb974ec567f6aa6b59d8a4653",
					"0x8ac725ae4f6948aab2d260defb21e2c77a8136f09887e36bfc1b0d208b8152bd0285cb4bca9ac2a4077e1f1eec8db53c",
					"0x83dfddfdbd4b86a4119af09a9a1869e3082d54a99bb11136d2c549794d76d6bae4672852d03f20d252cc0d4aebff2e46",
					"0x885408a4a536f0ed9aea262374557e0b1a407a7d047e56daa82d96c6118affbd83c8b098ebcca2b6d10727a5c87a2b44",
					"0x86036b83e3366469ccef4744de9f39fc899e37281423e35db79b32ce5133345483568215b14b673165f661f1a381661a",
					"0xb4b3981d5606b0c1f3c7ed40943d6fc9910efecba46391418f1b066c51d0d0370727308b21007e3bde09d23ef8df8d68",
					"0xb76370f3012034c17ea8c1714f498010bf24ecc3db369dd7eaf2ebc5d69b1d83781a455030603b9f39a24124b14ae9b7",
					"0xac618cfb012c7d912a54260fc9e92eae2b6eb607aa83d77023d60b563b30f2093ff7ffe506c682d9a3a77a1c5f88a4d5",
					"0x93c2264099643bc49e2eb73756caccd2dc9a5e81569f673eb6d4b6496e1672467e6324e32f6896fecc49a295c0a0907a",
					"0xae07ef469977cb7875b57b18b6f6a3a7fe4a9e934e61f806e13ee96ce863591cf8121974890be53db8085771f529969b",
					"0x805b68a262ec61bfed34a962ea624cc8200a8bc6424cd34a68bb09b14567d2dd9ad4725249001492cae84a274039e723",
					"0xaf6e0983c9565209dfeac2282abf498b8a0d8708b308b0fe0571a35913e67ca4f6700b8376306fe1c21217e53ed976cd",
					"0xb78c60b77d5315e97b76c671479c6195453abd3bfc8b111bc220d7698f9229eb75bef57d0bd857a7b0c4993c2f98a940",
					"0xb3a4a02939aad45db2e6bcf6ad1e06f68bc0130c22b22510fbd253d29b7001bf12db8afe3027183fa6236d146b5610e1",
					"0xa5b0f0f7b9a549235680fb5a65e3dc74f8a4300edd070615fd69eb1f37f0bf86048c3462705f9cf3b183aec664355776",
					"0xb1ec5fcef92047c2b29a88846e5a59cd3a877bb3d39c3955cb1a9e13a488d51340a89ab0f365c805cecd94d8e9722d90",
					"0xacfce7cc29991046205149184804157a9698beb2bf30770e887786c292fc7647895497d6651fd4085435a185c11dc71f",
					"0x98fe4a23246e01e93066229819a79089cab161323208432366760b787b745736c2ef802396de4643f331f380b306f41d",
					"0xa64d180aa02b04f220245a6c8ee9987063b5080be000b27cec49bf2f549f083f4977e5a41ed1da0780fc397e7d433363",
					"0xa98bca75c90d548ab712fa49cb0e1a1c91598abb91866d4ed7d5991e169df7f6f03df4666d1013b07e77be4a1f975a8e",
					"0xa48d729e1c57b5cbcf5e7f73fc4c2377536d8b1996f5f8f1e27b41747654c20dc0a927b95c3b492bd99d36ef27719723",
					"0xa315cee9a3d482f16e9297c4e1d942e1728000ab7e7b64199e7d0d4c7ff9a8341cdf295b619c7acac1bc42cc30958d56",
					"0x994b827e04a3d317f203a877ca86e2b9725055272392ab0ebb8d07d76f61d3f33c1168d717ca0f7d74029bb07228cbfa",
					"0xb990c6017f5202498300bd4c141ab07ca246ced9fe6fef80d6d36ba2159a98249137c2b7f4f34d60188997aa78a8dd41",
					"0x830afd3199936bf85063418119f50a823ef2fde9bb329897c94c5a83a20a910d4843124b8039e6fa6f97cf82d453596f",
					"0xb8fdb223b69c6f0d6c920e67b510973f7fb0be7f3c5e20c88a98bb82e44677e62827ce2ad8fb3e741a7cfd99ebb37c80",
					"0x8de6107e84462cce3ece3e6dbc3e484be5a7b57cfa2c93e4e4cfd64fea6b77c9a3a573170db5eed02f48e2047b199743",
					"0x820b22600a258c6a992138f62e2979398ffda67f7bad2cfa8be98185ae7c0ff69c492d1bbd8f31033140cf4d1d50f616",
					"0xa051710883f5eeb5fb29b08e7945718b5002a9589db4a38eeb25780969893fead9eee2b2d5a607dff1d306668f5c96e6",
					"0x8114028a459e1907bfb1260c0cd113b706309ea726abc22fb06a121415f9d3ec54bd9ea4997b458f09f21edcea7277b6",
					"0x9037df9cd765f2526e353c2c11c3ac1f0c1e2317c135fc8c26ca2f51d0e3dad8ab64419308fb8a8e5576b87238e16ec3",
					"0x957a04f010099692e2e60801586ed3b1d227c05b58dc0d7df1b813376e8a65342533363a9c2fd4ce0b7e238b530075d7",
					"0xa116da8b6078e92c7ceb3fb96b22a4158e7e3022c4f0a5fa7fefa17853fd3443185e469897f0ed68d7e25d001dd5c809",
					"0xa241eaa7c57fb24f055a266de6f3d7a0d590a0275d3ec5b80c79c15c65ad30f553c19b1326dfcc67833f8d81ef66787c",
					"0x8e05afba4b6632b65b59d4c01551f3b05ca6979322ccbee97099a74c6ef0ca6a3937899cf13e6f481072aab88f3c100a",
					"0xac7cbdc535ce8254eb9cdedf10d5b1e75de4cd5e91756c3467d0492b01b70b5c6a81530e9849c6b696c8bc157861d0c3",
					"0xb96e6f17590c2045529a38fc06290e45b1c60848250b7bc1fbc8b88f1a6b269fe7c3557889baa0d06daea66ae183edac",
					"0xb5374a63c5942dd76ef30e4e223532b35b92b294f4d640727165042577bb5c6bb6621e657c46397b026d9d5a31be2fc5",
					"0x8190762aa446b841b766db896f28303942cace7b87f246334254329a5c07ca0507c4db92ae102db06d3e00c11706cdfc",
					"0x97cb345b7b0ca3f5aa1f500cd9f6b6bc6d7352467341fc12226f030f940063ad8bf5b6876c82a592d4b6b995e79085af",
					"0xb11eb70da3d1cea5377b60c581d6a9f2d6f6bff9e7fe06536dee3aff99b600953315b70b59f12da5ac75554d6d29ee12",
					"0x881c6056b9776d413d9094ac6814dd67a94c118f34e0be2d02a0cbc9466356d8f3b710ee4bab1903eda94e46c12ff954",
					"0xb56a1a967f470379e466a2b6e3e7feefa8f388fb1ad925a9edadbf6c076d92aa236e5d6b278a94f0cd3159726e4ff40b",
					"0x90987c14180f0387ae58afa36535be44458cbd22f0b7a2a668c4459572e0150a0ad4b9785b3a35864f21576faf5dab9d",
					"0x8a51d4472ad2107f5be5b5e04f5ffb25e1e6cea60de176454ccdd5f7fc95f05a2f59af05c804f331925bb4a458ccc4f8",
					"0xaecdff2ffc46f703de2a43c07ee6b6d0d647372b2c1059a649d14b6b4f11e93ebb00d06a50636831a60fe2aee8c9191e",
					"0xabcb2969d1289217bdce6f8e37e56f511927278e829d7387788ba5cb3e7be3b8dd3b9e6512e8e5db20e7b70b531d5e6d",
					"0x8128a6fd2d827cad78cd07b40da9d6312d14fb36b18e89ad9fb018ac1d040202e9dbf9fd327502a61993be40eefd1709",
					"0xa92f485ad3b24de58117d326851abeb6333ebd46fbf4862b06bad17b0aa869c634d2d4e3bbf8f8b19515e99fc98e7154",
					"0xb5b3c7a4b06b2d9ba6fc9d018cab4cd89358120d99eea64283b5ee6b150c4d27dbcfeadddc305245e08e6f8fde8ecd7e",
					"0xa3d5e60f20f89bc6a7b39562c13ed12b8ac5b8a0bb365f68e5c32161c7baf4d8e79beee608571692878778cdd4b523fe",
					"0xa63c779a6eba9fc642fc8da55fdbe2b9d84183bec0b8e7cd5623ad5c9d8d63750347b98f82f5a60e628da8c3864a17ac",
					"0xa53d8bfb9654371be7410a4df6d296c2337be5dd020c365485b89f7ccbbe60806310d986aaf7a2d077e1e0133285c5ca",
					"0xb5ec1d36ee8aa5c4002f5d210605e41490e3bd5bd33e61606f3c33a9744591850985d9cdeda7313e1b7bcbe04b007122",
					"0x93dcf4f6e856e59db0aed87ebc9022d0eb8d84cbbde34f0251b304bb7cc677c0b9b42cf272764fa180600d64900fc3d7",
					"0x96acf46486ac90445ac7b5ee2c44f76c2c4b087efb1d37b32950f66f15bc0849bf9cf037492e82d6a36f97b091d9d763",
					"0x90a9f2d77e025829eee3f5c55ff05e6bd2205cf3e5da0466e3242db31ad72b039c32c18e6a71c6d919613c76ed6c955e",
					"0x8566b4a86d4ee9534b8324d5cd13c6b58316a09883e4772232e03bf0c4dc45811717bed41ba2cb2347abbec5a46cb428",
					"0x8c9d26efa20dea2c0b17a9fb11297c3ebbb8bc52ae00072c50033ea54ec34add5fa3d5e585e9f4e747aa7567cbd89a8b",
					"0x893f4329effebb9b0cf2f402fde1e38b48ad1a29564f35f0b66e764f7b11b8ab8abe3c17b7de847e4f3583c6b884e1f9",
					"0x84b16bfa0df9a136564e71f2740a957970c07f7854611bdb02b9f6f584f5c6c5bffd30199da4e4fd633e9e495464b659",
					"0xb205957b87b9ca61a88b4935e7bd5bc4991a82b3f1e00f36441c5a5ce79197b5bdebd280e722f97478a39766485cf763",
					"0x95372e2c6e3c513eb1911504be520164fbf14b3d9e763ddc2d2f26b45888972d60ab5a517c9903148256864ded2e98a8",
					"0xb8f76a5a4558dcc9c4e894674d8e7f70042710be64e9dc5cec1819da0a54b9211fc234526a276ff9ddca37bb95b61bfc",
					"0xa85b840c597b2fc2760cc13e3d023e940112ada907b21e9ed47e7cb8fba34daa80c699bf5c082b3502b2dc62470a3660",
					"0x85f95f3bed3bb789e61c5f06363afb3af623d65932a24703ddf93e9fe8caef87fc4b3ac6267956a6cb5962f637ee4b24",
					"0xa39e350f7dc28e0c4a6eef05856d6d16914ba81344b85d26d1dbac3b7b6b6051d90aab3a6c0d67e12eefeb95708892d3",
					"0xb8b19ed711950693d79aca5dd2baf991568701c0b04545a2e242554c17be9f90254de08b26bb184a12eabfb203ebebfb",
					"0xb7da99778f15b52fafa664d8fc6a0061d95c56fe2a6bd02f63ddee9e3781abae716b2ad26ccd0f5d7c25a2e4f1fcdfea",
					"0x8f4de543bb1eec03bd0e24d8d469b18ef9707c0599bcc7e7d2f675f2fc416fb7e4e8464c43b740a7450d434b76e5d7b0",
					"0x8d1eb76dba8ad2fd7762114f39e5f04efcb8257cadf3f95823d89e2e3c83a075f8fcb18773a075755be372a1e914543e",
					"0xa2e4aa140fbca591dacc1bbd6e47af591a16572659bb81aa0a5c5307909c91758d872db6eddb9c27e04797d830834ec4",
					"0xb52444f7d5364da09e4bb6db1f406568c1712eda4db5eaa257102497f8fb46661fe7bb5093102691617441110717294b",
					"0x98675bd1ac20c6f79090bd971f19fd3ac3b88d066272dec17e688edff0fa20baf515b94b43c56ed314b7b5f9b5603b5d",
					"0x924886581da5b722e10c3d545000df0bb5abf10baca34734e8acb49b1368dea9cb9b9d54a81b77f0573a3f5e32ae5b61",
					"0xa26080be5fe40558a50941b52c7e63e7f4498adc5778434719e9dfb5d405b0ed942b29cf2f029e689127d7a628ec635a",
					"0xa147fd86e1b0990ff6f0c4b9ea4d0c1441fcbe97964317b9b8ef2be22c601fd8445d3e349476874e03fb4dd60b6d7545",
					"0x977e1349d9a59ee779ec52611d99bbf2d1f29fcd84f5b4d46447071659835d30c8b6fc28a6e26040d28b453a5c47c904",
					"0xae1277663a164d8794ba8257a051dbeed8637eec6e43686c1493eab6ac78ed246b612df16fc07d4eb0b6b6daeb142748",
					"0x8f4607f08f479ee5f8975acacb7952fb8096e3fe1d344f9ef3d259240c27bdc4527b519ecddb2d8ab96c792280baa524",
					"0x90a3a0d984cc45f29123fc68b9f5e91cf50824ae2768f8cb3d11fe6f48f772c74cc634cf62539fbfa699313f3e3619ad",
					"0x999066753cd1e95b93384193c5efc67b1ebb966944d643defebe69c3e50a8d3086901b73da32f8b6c793536a1cc3cc0e",
					"0x907b7bd802b6f38cd8863db79c16758a308a1f2fc70e2cb5339effa1fa64d63b4eefb9ba8ddfc893fff5804c0aa55466",
					"0xae2dccc4da97c99f501ab3ded9eed3080c752fd224cabdde38dd5e41b2667b7867bf51b45814a2f70027ec7301fe1624",
					"0xadfac34fa0ed6e7e66aea814ab1dcd86130281632a86acd1d46c3f62417dc3ca90ff5718d15e7b610181434d5015c845",
					"0xb94bedf17c6d6580aa3ddaf0ea772d32ed3c8ce9a4804daa3d27c30fccd295e989dcb0d42ced00c9da29ce73c6963409",
					"0xad3821ebda52bf25606daec5d8be5a4c5d6f506d4d4d4f4546ed123f6453434ea5b6d746f63376faeec1ed4e678a2f70",
					"0x8fe563b6700ce112ff9715dfddb0a5f895a2e491e085d6b55a725a31c15d9435f9aa335445f9f397cb9716fbb19c81e5",
					"0x983874f99c786f5afc57d051919fff9bfbddd1f7bae7fbeb055e6541a371cf60bfe5c8d9eaed6693407daaaf42cb09a2",
					"0xa66c27d74e19fe8f3c89e472e0053b50b4578efa70c5d16c7b0efabc5b9dd5200d9d5de9c1d770fb7e9aa69069ebffdb",
					"0x907c5014952a9e2645fbff9f6dbb8cb64960443d22461606d914b68fb155d762a707a2d9153981cd9a29368e61bc9cff",
					"0xb1ce28491e1e8ce67d9abb6280d4de8e0627ba613019671ba7f8865358fb3693c83b054bf4ef50edf4d9d4f010d5ee59",
					"0x8c6c7711fe161033abc8004325ba479d781a254a85fcae6089755212dee66c5bcd532a7f99f3257b08fdedf37c3376e1",
					"0x94621761f7ec3638a35e8faf74b02d6baa4e1d081c2dcb42788ecd801a55791660e7560ea6077a8d17ea2b6a01756e0c",
					"0xab947287a6a493e75713bfe6303627d02f2ae1f3b01f138a78485278a2f72f6b5311bf3daa97758ea76017d38ba29b66",
					"0xaeb8b528d119a6474760c93482c8e42f832129dcea368ae17c4631295f516ea03f6968cbbb4b7c535671927625cabcbc",
					"0xa24cbb3644a02f765be18ac7b23bd9608b24ed0240f1a611ede691d91689deacfa00d80e3eef2dd24a12a1dfb15cc6a5",
					"0x8717883a49bb0503e5fa75283118ede9ec0a333cb6b96beaea801f16bffc98233f890bf5c6e003b66b0972c810c3d67d",
					"0x89d8e3819018a5c8f49721b634ba378c64c963dda1a4a954edce3819fe9cd8ee3b3253700451b89f360cfbf115d90e84",
					"0x91ec47cd8656aebba0391db8ac579e5e748bb4c7675bf94eddf4a6f5f9a9afeb15e72572890ac23ca6d76ab7d17064d8",
					"0xabed4bc22e76db7203ea3a9098f5aff12a3e6618173cda66daef0beb126c5cc5c98e78986ae98521719a1dc3457a12cc",
					"0x8562ddc3980407cc2c4bd3cc9fcea52e7adfa37b55083e36813ad84a6d0d74cd69879eaac747b5ef3e23422a0c0b4df1",
					"0xa4fde062eb5657cca941dd444e46596fe53f514106d46253ee00eade7de9a3179202caca12ba467713b268e847faf889",
					"0xa8f136f59ffbe9df0858eaf8d10c75446edcf6ad9a52b498451f9a0f855e6afd530fc5c807f4d6972c394474c4e07bb6",
					"0xa400fdd6efb6ac59b8e46570dd1d79c4f6805e218b43d35ffca4f4102818a9e43232431099771595a0642105a1d0ea14",
					"0x9942b3b3bde536d95d02bd5d0827f1035f046218e73b54f1f681d98ecee3aa05065c52e38fd8c8e9cc191915ff6ebe66",
					"0xb0a77d47a3b5fcb42c5fa2fd9b142ab0c61d3dde09057b14a7a025c9a5d7182ee3a008a574e62203562cf34ebb3ad5cc",
					"0xb7dbe24157a05b7e3dcf24684509beee0caee2b073385f8693bbdb81427127ca287d0ecb4f994538132781c11686e0fa",
					"0x8959fe20c49ef0b5a07d00b617605e743d2527102543a160933db66f864f51df8e7e2930c3a16ebf5520c470e04aa0c3",
					"0x806f4d77770d3014ef0c87a52ce285c7e7c8bc9c7d9f50b7a0a06593a02caee954b20b9506952502845791f93b8e1e22",
					"0xb7c4848f94cd54c92ff832779bbf3642ca94b364d7974a917678a8f7c6c441e04da57cd4bf4c126ae2cb69b22d0879e9",
					"0xb3245f0c1b9148ba255c27a35188e338888f16383d15f3715ee9721f74f2ea6ae83af27f2e405fb6267b6a3f70ddca93",
					"0xb63bff656f7e6994ebc4ae6c7bed5e6658d9895c06e5eeeed72deb6370933d214ae6c6c28a0690c9f945c72ada903187",
					"0x935122ecf23db6c4fe05e3db025a8d81ce66a9e4e7c2f863a0d68e7d781162f8282803640fc29c3b5f6ebd56958b439e",
					"0xb9392a99ff0dd51ebd221dd96f0e3276fce8d41c57fdf17efc65cc08ad62afbe79b9e31dfb7babc907a1c2ea9a91e15c",
					"0x80896769076d7ef0a2573ab2b9d2734ca6c41327681f6092bc2108dc0e391af465fa13ee2d019959919af9bb20e839bc",
					"0xa131c81f5fcbb58cb9d82081fc1f416708c367821c336f1058e9e15a84eed838af8b2c5dd75503019c497288812c48b4",
					"0xac938613ba1272613369df184187bc592a2754dac0a725ed9a957aa9d9a2e086859654062871b0f161c059a8bd1b8b57",
					"0xb2364b0becf1874283e0c9b9700d4ebe833c55c4df8b61d58069d89c085e39188c4291f92191d92dc4e5813b83fa986e",
					"0xac6511c9735257549a91a9d8f5500ba6e6e050c62e5604141e55f5ee644c3fa26d96f955c237c84d77eb2a19837bc250",
					"0xa178c32789d165e97e3af6f00a1af8a6fe67c911d887637d49bb452e6ee021aa337924a3a6c60131f66121dc47a698dc",
					"0xae6c675f343555aad99f90022dc8b35bc759d45329bdac6a8114c05a6dfcff066c14ab4e5b88a463c4f63fbdd2132213",
					"0x98e9f06fd4a2b19b376d852e2aa84de3773f7e67d8cdb96256c28ebc27e282bd4e98df33e191c9d4588876d1e27023f6",
					"0x86ff467740fc1deec59d3c5e966122bbd4dfa05783f9a79faea95849879c6a4812cf367c9920f975e0476a8f4b76ac4f",
					"0x8510e9a585bfed4a745641d493e5fcbc64e2a50c776e47d82069db2e4a89e75c89c0a464245755ce0ca5b93b14e3f910",
					"0xa3fd7fb1a7bea0cb2ac07cde86ea71798046607ed6aa4fd9d2f10b152dd86558374979cd6612c94c27be54d674cc00cb",
					"0x8e097a30b697480eb79c95754260e898d4fb5154f83765677009ef14bac6fccae63ededd1d7c2d1d5617f2b848f51475",
					"0xae43e4dde57a63fa1771bcfa3b0574036b595bd09abc3516f21567a7787175ceb872199c0ed6959f3accb662000d278b",
					"0x950379cba518cd11632a5443930903ca2147b9a7d20d2105b69a4c6d10166ae8c2685df45a0e830e506c2373e3230d0f",
					"0xb46af375c107e87aa491daa2d448f7be8519e0bbdaf37b3c2cc461998d412aa33600110d780d344c2d3e9efb5f44aa75",
					"0x8bd2e4a6ce9c23b4dc497425255907f6b650a07c61b68f3a1f27eb3333fa264771b318e4f20a177449588c5d8139a5fb",
					"0xa05752aa503b1fd9ca567fd6ee2236d0a903ff09438784ad256b5849b64e58b20ae08a0117589a6dc6a09bb6fd7ec761",
					"0xa2c8b2cfb4f63c8dbd6df0b71ba171e185fbcb076d432943241e193dbc3e5e645f5e48812a967aa17cb7f9e98f96731d",
					"0xb5b9e7f3010ca8cfcec4cd13c8791764f9b5470f1d369ebf9aff98925b54a8380f2f950f702e99e7894a98fd3e295409",
					"0xb920fceb402dd17f7e5453d9594f59c613e017186a253155412648133eb2f9e82096eb684dde1301a114f97cb0178445",
					"0x88e2cfe254f8969e79fdf99ad7adbbfea4f90a8afd6b7381f1279607bc33b76c02bf72ee7a588da36c9fbd531da60464",
					"0x8f4a7a520082dcabb3fb4c1165998289a07d7b2c9469f4dff8990fcd6c0bf82414ca7cf07f8a405f1bf149432f5442ce",
					"0xb0454cb4cba27e7d10e65f090726b77b5ff064690e9d721913b962213eb22a31a20bd96df198f8878611e1e9ac409302",
					"0xaf0ceaa3a5d22c5acf4c11a5cfd0a22c7f57df1600ee5b71eba18f98dff0543e097006083bf3bec74df17674f4e3bc97",
					"0x98573277c9d2746729ee89147b0bbf60fb0daa6a3fde17abeee9c961279fbfc48804621d6d785a0551f1ff1dabee0f52",
					"0xb8a8b7eac778abc61246b5392064b81894e7407dae0017c1f39b4d73bd1cd9cfe180728e5579c836ddec1ba7aee46c58",
					"0x929bb225f16ab6273cc32232c9d0d9e47519dd0a53a7c8cd58eba5f0cb116be4985dd768277b714074dfed27e1c00971",
					"0x97bef17ef4798c5b088299c563062c31ad5e0c5b6b14a7deb428a6720bf412bab04223c529b28ba7cec936eff8df446a",
					"0x822b8dc105245df22ce6c7fb9588b35015d874df945de66e7f4f364b26295c48bb274788e54ff115da452e1c9a4c02a4",
					"0xb9264d17e63fcdf05785e1180a5965619af4f70259b908f25dc2d57d4f73d086f89a1c9c5cda61ceef03906846d41cee",
					"0xb1eae9979a83277a480654d669426cc4369182b90e5271ce9c2a237879c262d8f23d7d4902352051852a5e9a8fadae33",
					"0xb7372bf08478db2cc047fef2f96fb2d762423d68b1a5c140e03dff8193da69891643861da5fdc59b57b1a2d4d295a916",
					"0xac13c8e4b2f3664a636babab9aa7240ee47d7fe94a664190a89248bdb5a4eef07e31b608dffe3d5e3abcd718d8bf2ed4",
					"0x99166649beda540a756a2028ddba21f1bc8556bd06ebdc4d495d7f0398156650d84eba656ab1ed82e3f4257af2fef5d6",
					"0xa3e5a3c7aafa419d0445c6eb5a9629f1760643e9eea8a7836c2c3f84fe8bad3bf0e805484dc133749854d57441bedc84",
					"0xab6095aa694579c046b55d563f272c8cec1a322b6ffac937689c755705d375cc1ec84bd4808e6885efd3dec5a7c6888a",
					"0xb2cbaf9027bcf485b3922c23d0a3cde1de1d22d78b56ff6c5edb813dd4eb4b7791413e67763cab52fa5e119fad401fe4",
					"0x8fee644fa7a87f4b4998f9ce3d29cbf372db16d65b774608696ed69d607c58ef137f7dd71bda2ac923ca26e86c08d787",
					"0xa5e8f4f07ad96c480d5f83fb826f4c44fe6d41317d3c57fce865e637b1222a6e447e12beb76ccfd1fcc1ce0eed3dde82",
					"0xa81e7ba5796d5951cf70216e1af93a6fe7a82b7af209e3bfbc2783a7dbc3854a15699f1da3d29e5ffe5fe5a08fe3c977",
					"0xa1f9001e0989c9c2bc742e397f7acf2f1c427f544f963af9c2cd17bab6729f656e4419a9c51359251d66b51e7a33fdc8",
					"0x957133ba4be47a6b7fabf2bcca19ec62c3edb2e3d484216a0ba32637df163a8aa94a05cfed66e1392a6c127625f5aaac",
					"0xa0c9df5fd32f1533d5522982e3af5e568dc7f860c1cfb9d2aeb725ce5fe6f878a8be82177ce00a64dafcfddddb5d6d0d",
					"0xb0f79c525acf07f813c2bc6f481c0606932aebfa509b900911f4f933b4ced1717be4f46f94f2d942c32fc928cd0ff663",
					"0x92baceaa0cc38a45a22ba42a531d8e8558e4595657cdfb2a96b64f09252d5ea58fb6759a8db218d8e052f8a3a8b83d31",
					"0xb901487c55a63b03cd2e647a47334c9fb870c3b2d7b7b3ef8a5fb0b24a9fe97feb5736cf390c0ed5b2b7124df447daba",
					"0x97861a7c48211727dbd232e6a1ce610f0ebca880c53ad5cbc1af282952583218a6a7db6626772146c99965b5d48c6770",
					"0x8d5bafe21c93bcccad8964e9e2976343cb979d047b2b8388ee589de6a469eddfcbc15b0b7e82895afc5c4b4dccff5bac",
					"0x8622ec7a5ba8bb99c829edcb7f100a91638a5ccb5e01057c8c4fcf44e7d14897fd3fac8a18391324b8953ffdf1b88820",
					"0xa774fb440bc178b30181bebab373d963625cbd097348957ff529f00a8bc4d835e6f4a626ce3a86529302e83283699736",
					"0x931919450b56860104bf3c208a5beb30446c326e89905bf1734db1157549c8820bb1fd5183a2dd413541cc92db116a33",
					"0xa5000a546d6a476fb8dc7e91e40827fc2d118ddb3661350e5d006bc468ac8058c45c078f4e88704548db00be3d0ead7f",
					"0x992b008fceb25875be9b7cd3ce843550aea19ae8b697c21c6e8bfc1fb4a060265db3cb3233339a8c8ffd2f55e7632ad3",
					"0xb9ae7ee9835c295986e8db6f0a72c997475e9ce20afb216381cd4f9231968297904bb35c02873dcf12c2a2dce7d00fa9",
					"0xa2345e1c3813adf951a4d15b42f31c847e63f60cf485ea3669210f6400a6b7290e4835068b83b640a3a09ee4a9c690f7",
					"0xa307fb31c932be1e734fd5b227739fe04f53a7aad6a0b5657c2b861c32346f68411778a27da8e04557c118b4659bebd3",
					"0xb6d114a91f1b74dac741af93ce684ebcffd273557bc1d3e0cee5a54a8afcd297c6f35e9df62f92b7f4b74f739600c393",
					"0x8b05b9fdccaa061673477ea452016111c5d3678aca09b339ecdef49f47b304d5a22f0f395bda3d320f93bc6e57bbb3ac",
					"0x85de5e8d90464ff655d8b529a2716692c4680c85b44fad88f62749b5f1fa93eb3e5bc41da7ed937ee98c6e9259bf296a",
					"0xad231d63548677ab098b75c441e05995b00d3184d4cbce48dcc6cb6184597da550fb1057ee358a28e5956702cfbfb798",
					"0xb1a9333022d690ce01f6bab3d7e7b39c00dbbbb5afdcaa05bf2d91da3b3c949bc458ba40d3a0e9d26cf918da2d41972d",
					"0x846657b8e56a67010c884fcbf8629357030e40a34e0441ec5d03eb99e9de31f17cb5e8c429c8a731a1afc87d2aac92da",
					"0xaf383d267f4476aa0079f6826bb085f0258a30fe3ad853292b641726b2e5ce0d715a763fb24e7a0718d4bd444814d0d7",
					"0x87fecc460fb0e2faf1dc7213ac0c6f551758833db68955cb157a6a14fd3938d549b0aa84339b135955789ef613b0d585",
					"0xb27efa360e590aca1c0a878b65e642444cac6f35dff8ec838fde6b5a78a952f3752076100b902ad4e152bb7e6e697888",
					"0xb5ec69fb5c52fb979bd3ec81c20d1cb54206e790086d97a60612bb708963c9de752bbc5db77355e46c910744b3a9f50e",
					"0x97508da50299f735f4ca32df1b906329eccf7a62d867bcf4246128ea3394d2e657fd9e06c51d6d00e811ca93f8911c29",
					"0x925b65758625533c1e40d141512c6dffc67eac2fa97e8a66a727cf2830f4980ecf093ef2a7c92bac861b4d33ed3d05ab",
					"0x91d3ef9e0553dec9153f48e3fe74ca272752bb0f0625234e66765fe8604e4328b9a6c3cdd4bfefe704eeaca29491bfb4",
					"0xb6612554a6ae3219aace98b29474959004b84475df210583bddd4a45b63c14a241e2799c3096cbac3bdd2c4db7ebbccb",
					"0x86f726122dfb35212c3dc0794a1498b4ac6509af9cd00eb7d7340ae2a78c9bd87bf9373d73972c01697ce30a7bf635fc",
					"0x8dadf757a7b8dc6c5916ef2ac3dd768d0e6dd80d429be1db40e46ac4ac63523bacb36e62eaf26a4b1622c19fdc694b80",
					"0xad7369bc2d557d35352fdda67ea412796469f24294cd6276c974f21a79bae13b2321d740c511cd0c16d6e032e6176b9e",
					"0x93f02fce0190dbb9ddc52e7582b8f00e69d67591d7c701c280df8287f1f54511bbf49df863e62b270306ee6d49fcebcb",
					"0xb885a9afb61a4bb5b9751e8d89759b81ea2b4af447496de350ef5f4c5aecb3280f0202535b194f4c7eb28083ad0b6800",
				},
				AggregatePubkey: "0xae2d00fd0b192ec2e792080a01b6a101b5d293d02e7643c22ca7353837c6c704a145f05e95577817151d9bb4b1b55ae7",
			},
			CurrentSyncCommitteeBranch: []string{
				"0x0b68206adf70650dba3c1d71e5e81fd473f72b601b3f2fdc83d086e7d504eed2",
				"0x5ee1b6bcdfe5680ed9d593cfb637d43177d9424de2ce9208313d4571630c7ac8",
				"0xefa3d52536d23cff74451ffbd3735e00351ad3800392f513b89783d7b671a6de",
				"0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
				"0x2c287827800c33e2f309a5b10237eaf839b42a1d982338b933e44e3d593752a8",
			},
		},
	}, nil
}

type Proof struct {
	Root struct {
		StringLeaf string   `json:"stringLeaf"`
		Proof      []string `json:"proof"`
		Index      []string `json:"index"`
		Value      []string `json:"value"`
	} `json:"root"`
}

func (s *Sync) GetFinalizedCheckpointProofs(stateRoot string) (Proof, error) {
	req, err := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/lightclient/proof/"+stateRoot+"?paths="+url.QueryEscape("[\"finalizedCheckpoint\",\"root\"]"), nil)

	logrus.Info(req.URL)

	if err != nil {
		logrus.WithError(err).Error("unable to construct light client proofs request")

		return Proof{}, err
	}

	res, err := s.httpClient.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return Proof{}, err
	}

	if res.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(res.Body)

		logrus.WithFields(logrus.Fields{"error": string(bodyBytes)}).Error("request to beacon node failed")

		return Proof{}, err
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return Proof{}, err
	}

	logrus.WithFields(logrus.Fields{"body": bodyBytes}).Info("body")

	var response Proof

	err = json.Unmarshal(bodyBytes, &response)

	if err != nil {
		logrus.WithError(err).Error("unable to unmarshal fork json response")

		return Proof{}, err
	}

	//logrus.WithFields(logrus.Fields{"body": response}).Info("snapshot")

	return response, nil
}

func ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func ComputeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EPOCHS_PER_SYNC_COMMITTEE_PERIOD - (epoch % EPOCHS_PER_SYNC_COMMITTEE_PERIOD))
}

func ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func ComputeSyncPeriodAtEpoch(epoch uint64) uint64 {
	return epoch / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
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
