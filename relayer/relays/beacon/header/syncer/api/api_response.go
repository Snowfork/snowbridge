package api

import (
	"fmt"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	beaconjson "github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type SyncCommitteePeriodUpdateResponse struct {
	Data struct {
		AttestedHeader struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"attested_header"`
		NextSyncCommittee       SyncCommitteeResponse `json:"next_sync_committee"`
		NextSyncCommitteeBranch []string              `json:"next_sync_committee_branch"`
		FinalizedHeader         struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"finalized_header"`
		FinalityBranch []string              `json:"finality_branch"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
		SignatureSlot  string                `json:"signature_slot"`
	} `json:"data"`
}

type BeaconBlockResponseData struct {
	Message BeaconBlockResponseMessage `json:"message"`
}

type BeaconBlockResponseMessage struct {
	Slot          string                  `json:"slot"`
	ProposerIndex string                  `json:"proposer_index"`
	ParentRoot    string                  `json:"parent_root"`
	StateRoot     string                  `json:"state_root"`
	Body          BeaconBlockResponseBody `json:"body"`
}

type BeaconBlockResponseBody struct {
	RandaoReveal string `json:"randao_reveal"`
	Eth1Data     struct {
		DepositRoot  string `json:"deposit_root"`
		DepositCount string `json:"deposit_count"`
		BlockHash    string `json:"block_hash"`
	} `json:"eth1_data"`
	Graffiti          string                        `json:"graffiti"`
	ProposerSlashings []ProposerSlashingResponse    `json:"proposer_slashings"`
	AttesterSlashings []AttesterSlashingResponse    `json:"attester_slashings"`
	Attestations      []AttestationResponse         `json:"attestations"`
	Deposits          []DepositResponse             `json:"deposits"`
	VoluntaryExits    []SignedVoluntaryExitResponse `json:"voluntary_exits"`
	SyncAggregate     SyncAggregateResponse         `json:"sync_aggregate"`
	ExecutionPayload  struct {
		ParentHash    string               `json:"parent_hash"`
		FeeRecipient  string               `json:"fee_recipient"`
		StateRoot     string               `json:"state_root"`
		ReceiptsRoot  string               `json:"receipts_root"`
		LogsBloom     string               `json:"logs_bloom"`
		PrevRandao    string               `json:"prev_randao"`
		BlockNumber   string               `json:"block_number"`
		GasLimit      string               `json:"gas_limit"`
		GasUsed       string               `json:"gas_used"`
		Timestamp     string               `json:"timestamp"`
		ExtraData     string               `json:"extra_data"`
		BaseFeePerGas string               `json:"base_fee_per_gas"`
		BlockHash     string               `json:"block_hash"`
		Transactions  []string             `json:"transactions"`
		Withdrawals   []WithdrawalResponse `json:"withdrawals"`
		BlobGasUsed   string               `json:"blob_gas_used,omitempty"`
		ExcessBlobGas string               `json:"excess_blob_gas,omitempty"`
	} `json:"execution_payload"`
	BlsToExecutionChanges []SignedBLSToExecutionChangeResponse `json:"bls_to_execution_changes"`
	BlobKzgCommitments    []string                             `json:"blob_kzg_commitments"`
}

type BeaconBlockResponse struct {
	Data BeaconBlockResponseData `json:"data"`
}

type BootstrapResponse struct {
	Data struct {
		Header struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"header"`
		CurrentSyncCommittee       SyncCommitteeResponse `json:"current_sync_committee"`
		CurrentSyncCommitteeBranch []string              `json:"current_sync_committee_branch"`
	} `json:"data"`
}

type FinalizedCheckpointResponse struct {
	Data struct {
		Finalized struct {
			Root string `json:"root"`
		} `json:"finalized"`
	} `json:"data"`
}

type SignedHeaderResponse struct {
	Message   HeaderResponse `json:"message"`
	Signature string         `json:"signature"`
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
	CommitteeBits   string                  `json:"committee_bits,omitempty"`
}

type SignedVoluntaryExitResponse struct {
	Message   VoluntaryExitResponse `json:"message"`
	Signature string                `json:"signature"`
}

type VoluntaryExitResponse struct {
	Epoch          string `json:"epoch"`
	ValidatorIndex string `json:"validator_index"`
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

type BeaconHeader struct {
	Slot          uint64      `json:"slot"`
	ProposerIndex uint64      `json:"proposer_index"`
	ParentRoot    common.Hash `json:"parent_root"`
	StateRoot     common.Hash `json:"state_root"`
	BodyRoot      common.Hash `json:"body_root"`
}

type Bootstrap struct {
	Header                     HeaderResponse
	CurrentSyncCommittee       beaconjson.SyncCommittee
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
	slot, err := util.ToUint64(h.Slot)
	if err != nil {
		return BeaconHeader{}, err
	}

	proposerIndex, err := util.ToUint64(h.ProposerIndex)
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

type GenesisResponse struct {
	Data struct {
		GenesisValidatorsRoot string `json:"genesis_validators_root"`
		Time                  string `json:"genesis_time"`
	} `json:"data"`
}

type ErrorMessage struct {
	StatusCode int    `json:"statusCode"`
	Error      string `json:"error"`
	Message    string `json:"message"`
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
		FinalityBranch []string              `json:"finality_branch"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
		SignatureSlot  string                `json:"signature_slot"`
	} `json:"data"`
}

type LatestHeaderUpdateResponse struct {
	Data struct {
		AttestedHeader HeaderResponse        `json:"attested_header"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
	} `json:"data"`
}

func (h *HeaderResponse) ToScale() (scale.BeaconHeader, error) {
	slot, err := strconv.ParseUint(h.Slot, 10, 64)
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := strconv.ParseUint(h.ProposerIndex, 10, 64)
	if err != nil {
		return scale.BeaconHeader{}, fmt.Errorf("parse proposerIndex as int: %w", err)
	}

	return scale.BeaconHeader{
		Slot:          types.NewU64(slot),
		ProposerIndex: types.NewU64(proposerIndex),
		ParentRoot:    types.NewH256(common.HexToHash(h.ParentRoot).Bytes()),
		StateRoot:     types.NewH256(common.HexToHash(h.StateRoot).Bytes()),
		BodyRoot:      types.NewH256(common.HexToHash(h.BodyRoot).Bytes()),
	}, nil
}

type WithdrawalResponse struct {
	Index          string `json:"index"`
	ValidatorIndex string `json:"validator_index"`
	Address        string `json:"address"`
	Amount         string `json:"amount"`
}

type BLSToExecutionChangeResponse struct {
	ValidatorIndex     string `json:"validator_index"`
	FromBlsPubkey      string `json:"from_bls_pubkey"`
	ToExecutionAddress string `json:"to_execution_address"`
}

type SignedBLSToExecutionChangeResponse struct {
	Message   *BLSToExecutionChangeResponse `json:"message,omitempty"`
	Signature string                        `json:"signature,omitempty"`
}

func (h BeaconHeader) ToScale() (scale.BeaconHeader, error) {
	return scale.BeaconHeader{
		Slot:          types.NewU64(h.Slot),
		ProposerIndex: types.NewU64(h.ProposerIndex),
		ParentRoot:    types.NewH256(h.ParentRoot.Bytes()),
		StateRoot:     types.NewH256(h.StateRoot.Bytes()),
		BodyRoot:      types.NewH256(h.BodyRoot.Bytes()),
	}, nil
}

func (s SyncCommitteeResponse) ToScale() (scale.SyncCommittee, error) {
	var syncCommitteePubkeys [][48]byte

	for _, pubkey := range s.Pubkeys {
		publicKey, err := util.HexStringToPublicKey(pubkey)
		if err != nil {
			return scale.SyncCommittee{}, fmt.Errorf("convert sync committee pubkey to byte array: %w", err)
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := util.HexStringToPublicKey(s.AggregatePubkey)
	if err != nil {
		return scale.SyncCommittee{}, fmt.Errorf("convert sync committee aggregate bukey to byte array: %w", err)
	}

	return scale.SyncCommittee{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: syncCommitteeAggPubkey,
	}, nil
}

func (s SyncAggregateResponse) ToScale() (scale.SyncAggregate, error) {
	bits, err := util.HexStringToByteArray(s.SyncCommitteeBits)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	aggregateSignature, err := util.HexStringToByteArray(s.SyncCommitteeSignature)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	var syncCommitteeSignature [96]byte
	copy(syncCommitteeSignature[:], aggregateSignature)

	return scale.SyncAggregate{
		SyncCommitteeBits:      bits,
		SyncCommitteeSignature: syncCommitteeSignature,
	}, nil
}

func (c CheckpointResponse) ToScale() (scale.Checkpoint, error) {
	epoch, err := util.ToUint64(c.Epoch)
	if err != nil {
		return scale.Checkpoint{}, err
	}

	return scale.Checkpoint{
		Epoch: types.NewU64(epoch),
		Root:  types.NewH256(common.HexToHash(c.Root).Bytes()),
	}, nil
}
