package syncer

import "github.com/ethereum/go-ethereum/common"

type SyncCommitteePeriodUpdateResponse struct {
	Data struct {
		AttestedHeader struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"attested_header"`
		NextSyncCommittee       SyncCommitteeResponse `json:"next_sync_committee"`
		NextSyncCommitteeBranch []common.Hash         `json:"next_sync_committee_branch"`
		FinalizedHeader         struct {
			Beacon HeaderResponse `json:"beacon"`
		} `json:"finalized_header"`
		FinalityBranch []common.Hash         `json:"finality_branch"`
		SyncAggregate  SyncAggregateResponse `json:"sync_aggregate"`
		SignatureSlot  string                `json:"signature_slot"`
	} `json:"data"`
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
