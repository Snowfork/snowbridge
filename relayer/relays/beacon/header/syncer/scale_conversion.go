package syncer

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"math/big"
	"strconv"

	logrus "github.com/sirupsen/logrus"
)

type SyncCommitteePeriodPayloadJSON struct {
	AttestedHeader          BeaconHeader          `json:"attested_header"`
	NextSyncCommittee       SyncCommitteeResponse `json:"next_sync_committee"`
	NextSyncCommitteeBranch []common.Hash         `json:"next_sync_committee_branch"`
	FinalizedHeader         BeaconHeader          `json:"finalized_header"`
	FinalityBranch          []common.Hash         `json:"finality_branch"`
	SyncAggregate           SyncAggregateResponse `json:"sync_aggregate"`
	SyncCommitteePeriod     uint64                `json:"sync_committee_period"`
	SignatureSlot           uint64                `json:"signature_slot"`
	BlockRootsHash          common.Hash           `json:"block_roots_hash"`
	BlockRootProof          []common.Hash         `json:"block_roots_proof"`
}

type FinalizedHeaderPayloadJSON struct {
	AttestedHeader  BeaconHeader          `json:"attested_header"`
	FinalizedHeader BeaconHeader          `json:"finalized_header"`
	FinalityBranch  []common.Hash         `json:"finality_branch"`
	SyncAggregate   SyncAggregateResponse `json:"sync_aggregate"`
	SignatureSlot   uint64                `json:"signature_slot"`
	BlockRootsHash  common.Hash           `json:"block_roots_hash"`
	BlockRootProof  []common.Hash         `json:"block_roots_proof"`
}

type ProposerSlashingJSON struct {
	SignedHeader1 SignedHeaderJSON `json:"signed_header_1"`
	SignedHeader2 SignedHeaderJSON `json:"signed_header_2"`
}

type AttesterSlashingJSON struct {
	Attestation1 IndexedAttestationJSON `json:"attestation_1"`
	Attestation2 IndexedAttestationJSON `json:"attestation_2"`
}

type IndexedAttestationJSON struct {
	AttestingIndices []uint64            `json:"attesting_indices"`
	Data             AttestationDataJSON `json:"data"`
	Signature        string              `json:"signature"`
}

type AttestationDataJSON struct {
	Slot            uint64         `json:"slot"`
	Index           uint64         `json:"index"`
	BeaconBlockRoot string         `json:"beacon_block_root"`
	Source          CheckpointJSON `json:"source"`
	Target          CheckpointJSON `json:"target"`
}

type CheckpointJSON struct {
	Epoch uint64 `json:"epoch"`
	Root  string `json:"root"`
}

type SignedHeaderJSON struct {
	Message   BeaconHeader `json:"message"`
	Signature string       `json:"signature"`
}

type BlockJSON struct {
	Slot          uint64        `json:"slot"`
	ProposerIndex uint64        `json:"proposer_index"`
	ParentRoot    string        `json:"parent_root"`
	StateRoot     string        `json:"state_root"`
	Body          BlockBodyJSON `json:"body"`
}

type ExecutionPayloadJSON struct {
	ParentHash      string `json:"parent_hash"`
	FeeRecipient    string `json:"fee_recipient"`
	StateRoot       string `json:"state_root"`
	ReceiptsRoot    string `json:"receipts_root"`
	LogsBloom       string `json:"logs_bloom"`
	PrevRandao      string `json:"prev_randao"`
	BlockNumber     uint64 `json:"block_number"`
	GasLimit        uint64 `json:"gas_limit"`
	GasUsed         uint64 `json:"gas_used"`
	Timestamp       uint64 `json:"timestamp"`
	ExtraData       string `json:"extra_data"`
	BaseFeePerGas   uint64 `json:"base_fee_per_gas"`
	BlockHash       string `json:"block_hash"`
	TransactionRoot string `json:"transactions_root"`
}

type Eth1DataJSON struct {
	DepositRoot  string `json:"deposit_root"`
	DepositCount uint64 `json:"deposit_count"`
	BlockHash    string `json:"block_hash"`
}

type BlockBodyJSON struct {
	RandaoReveal      string                 `json:"randao_reveal"`
	Eth1Data          Eth1DataJSON           `json:"eth1_data"`
	Graffiti          string                 `json:"graffiti"`
	ProposerSlashings []ProposerSlashingJSON `json:"proposer_slashings"`
	AttesterSlashings []AttesterSlashingJSON `json:"attester_slashings"`
	Attestations      []AttestationJSON      `json:"attestations"`
	Deposits          []DepositJSON          `json:"deposits"`
	VoluntaryExits    []VoluntaryExitJSON    `json:"voluntary_exits"`
	SyncAggregate     SyncAggregateResponse  `json:"sync_aggregate"`
	ExecutionPayload  ExecutionPayloadJSON   `json:"execution_payload"`
}

type HeaderUpdatePayloadJSON struct {
	Block                         BlockJSON             `json:"block"`
	SyncAggregate                 SyncAggregateResponse `json:"sync_aggregate"`
	SignatureSlot                 uint64                `json:"signature_slot"`
	BlockRootProof                []common.Hash         `json:"block_root_proof"`
	BlockRootProofFinalizedHeader common.Hash           `json:"block_root_proof_finalized_header"`
}

type AttestationJSON struct {
	AggregationBits string              `json:"aggregation_bits"`
	Data            AttestationDataJSON `json:"data"`
	Signature       string              `json:"signature"`
}

type DepositDataJSON struct {
	Pubkey                string `json:"pubkey"`
	WithdrawalCredentials string `json:"withdrawal_credentials"`
	Amount                uint64 `json:"amount"`
	Signature             string `json:"signature"`
}

type VoluntaryExitJSON struct {
	Epoch          uint64 `json:"epoch"`
	ValidatorIndex uint64 `json:"validator_index"`
}

type DepositJSON struct {
	Proof []common.Hash   `json:"proof"`
	Data  DepositDataJSON `json:"data"`
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

func (h BeaconHeader) ToScale() (scale.BeaconHeader, error) {
	return scale.BeaconHeader{
		Slot:          types.NewU64(h.Slot),
		ProposerIndex: types.NewU64(h.ProposerIndex),
		ParentRoot:    types.NewH256(h.ParentRoot.Bytes()),
		StateRoot:     types.NewH256(h.StateRoot.Bytes()),
		BodyRoot:      types.NewH256(h.BodyRoot.Bytes()),
	}, nil
}

func (s SyncCommitteeResponse) ToScale() (scale.CurrentSyncCommittee, error) {
	var syncCommitteePubkeys [][48]byte

	for _, pubkey := range s.Pubkeys {
		publicKey, err := hexStringToPublicKey(pubkey)
		if err != nil {
			return scale.CurrentSyncCommittee{}, fmt.Errorf("convert sync committee pubkey to byte array: %w", err)
		}

		syncCommitteePubkeys = append(syncCommitteePubkeys, publicKey)
	}

	syncCommitteeAggPubkey, err := hexStringToPublicKey(s.AggregatePubkey)
	if err != nil {
		return scale.CurrentSyncCommittee{}, fmt.Errorf("convert sync committee aggregate bukey to byte array: %w", err)
	}

	return scale.CurrentSyncCommittee{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: syncCommitteeAggPubkey,
	}, nil
}

func (s SyncAggregateResponse) ToScale() (scale.SyncAggregate, error) {
	bits, err := hexStringToByteArray(s.SyncCommitteeBits)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	aggregateSignature, err := hexStringToByteArray(s.SyncCommitteeSignature)
	if err != nil {
		return scale.SyncAggregate{}, err
	}

	return scale.SyncAggregate{
		SyncCommitteeBits:      bits,
		SyncCommitteeSignature: aggregateSignature,
	}, nil
}

func (b BeaconBlockResponse) ToScale() (scale.BeaconBlock, error) {
	dataMessage := b.Data.Message

	slot, err := toUint64(dataMessage.Slot)
	if err != nil {
		return scale.BeaconBlock{}, fmt.Errorf("parse slot as int: %w", err)
	}

	proposerIndex, err := toUint64(dataMessage.ProposerIndex)
	if err != nil {
		return scale.BeaconBlock{}, fmt.Errorf("parse proposerIndex as int: %w", err)
	}

	body := dataMessage.Body

	syncAggregate, err := body.SyncAggregate.ToScale()
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	logrus.WithField("sync aggregate scale", syncAggregate).Info("sync agg")
	logrus.WithField("sync aggregate text", body.SyncAggregate).Info("sync agg")

	proposerSlashings := []scale.ProposerSlashing{}

	for _, proposerSlashing := range body.ProposerSlashings {
		proposerSlashingScale, err := proposerSlashing.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		proposerSlashings = append(proposerSlashings, proposerSlashingScale)
	}

	attesterSlashings := []scale.AttesterSlashing{}

	for _, attesterSlashing := range body.AttesterSlashings {
		attesterSlashingScale, err := attesterSlashing.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		attesterSlashings = append(attesterSlashings, attesterSlashingScale)
	}

	attestations := []scale.Attestation{}

	for _, attestation := range body.Attestations {
		attestationScale, err := attestation.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		attestations = append(attestations, attestationScale)
	}

	deposits := []scale.Deposit{}

	for _, deposit := range body.Deposits {
		depositScale, err := deposit.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		deposits = append(deposits, depositScale)
	}

	voluntaryExits := []scale.VoluntaryExit{}

	for _, voluntaryExit := range body.VoluntaryExits {
		voluntaryExitScale, err := voluntaryExit.ToScale()
		if err != nil {
			return scale.BeaconBlock{}, err
		}

		voluntaryExits = append(voluntaryExits, voluntaryExitScale)
	}

	depositCount, err := toUint64(body.Eth1Data.DepositCount)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	executionPayload := body.ExecutionPayload

	baseFeePerGasUint64, err := toUint64(executionPayload.BaseFeePerGas)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	bigInt := big.NewInt(int64(baseFeePerGasUint64))

	blockNumber, err := toUint64(executionPayload.BlockNumber)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	gasLimit, err := toUint64(executionPayload.GasLimit)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	gasUsed, err := toUint64(executionPayload.GasUsed)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	timestamp, err := toUint64(executionPayload.Timestamp)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	transactionsRoot, err := getTransactionsHashTreeRoot(executionPayload.Transactions)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	randaoReveal, err := hexStringToByteArray(body.RandaoReveal)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	feeRecipient, err := hexStringToByteArray(executionPayload.FeeRecipient)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	logsBloom, err := hexStringToByteArray(executionPayload.LogsBloom)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	extraData, err := hexStringToByteArray(executionPayload.ExtraData)
	if err != nil {
		return scale.BeaconBlock{}, err
	}

	return scale.BeaconBlock{
		Slot:          types.NewU64(slot),
		ProposerIndex: types.NewU64(proposerIndex),
		ParentRoot:    types.NewH256(common.HexToHash(dataMessage.ParentRoot).Bytes()),
		StateRoot:     types.NewH256(common.HexToHash(dataMessage.StateRoot).Bytes()),
		Body: scale.Body{
			RandaoReveal: randaoReveal,
			Eth1Data: scale.Eth1Data{
				DepositRoot:  types.NewH256(common.HexToHash(body.Eth1Data.DepositRoot).Bytes()),
				DepositCount: types.NewU64(depositCount),
				BlockHash:    types.NewH256(common.HexToHash(body.Eth1Data.BlockHash).Bytes()),
			},
			Graffiti:          types.NewH256(common.HexToHash(body.Graffiti).Bytes()),
			ProposerSlashings: proposerSlashings,
			AttesterSlashings: attesterSlashings,
			Attestations:      attestations,
			Deposits:          deposits,
			VoluntaryExits:    voluntaryExits,
			SyncAggregate:     syncAggregate,
			ExecutionPayload: scale.ExecutionPayload{
				ParentHash:       types.NewH256(common.HexToHash(executionPayload.ParentHash).Bytes()),
				FeeRecipient:     feeRecipient,
				StateRoot:        types.NewH256(common.HexToHash(executionPayload.StateRoot).Bytes()),
				ReceiptsRoot:     types.NewH256(common.HexToHash(executionPayload.ReceiptsRoot).Bytes()),
				LogsBloom:        logsBloom,
				PrevRandao:       types.NewH256(common.HexToHash(executionPayload.PrevRandao).Bytes()),
				BlockNumber:      types.NewU64(blockNumber),
				GasLimit:         types.NewU64(gasLimit),
				GasUsed:          types.NewU64(gasUsed),
				Timestamp:        types.NewU64(timestamp),
				ExtraData:        extraData,
				BaseFeePerGas:    types.NewU256(*bigInt),
				BlockHash:        types.NewH256(common.HexToHash(executionPayload.BlockHash).Bytes()),
				TransactionsRoot: transactionsRoot,
			},
		},
	}, nil
}

func (p ProposerSlashingResponse) ToScale() (scale.ProposerSlashing, error) {
	signedHeader1, err := p.SignedHeader1.ToScale()
	if err != nil {
		return scale.ProposerSlashing{}, err
	}

	signedHeader2, err := p.SignedHeader2.ToScale()
	if err != nil {
		return scale.ProposerSlashing{}, err
	}

	return scale.ProposerSlashing{
		SignedHeader1: signedHeader1,
		SignedHeader2: signedHeader2,
	}, nil
}

func (a AttesterSlashingResponse) ToScale() (scale.AttesterSlashing, error) {
	attestation1, err := a.Attestation1.ToScale()
	if err != nil {
		return scale.AttesterSlashing{}, err
	}

	attestation2, err := a.Attestation2.ToScale()
	if err != nil {
		return scale.AttesterSlashing{}, err
	}

	return scale.AttesterSlashing{
		Attestation1: attestation1,
		Attestation2: attestation2,
	}, nil
}

func (a AttestationResponse) ToScale() (scale.Attestation, error) {
	data, err := a.Data.ToScale()
	if err != nil {
		return scale.Attestation{}, err
	}

	aggregationBits, err := hexStringToByteArray(a.AggregationBits)
	if err != nil {
		return scale.Attestation{}, err
	}

	signature, err := hexStringToByteArray(a.Signature)
	if err != nil {
		return scale.Attestation{}, err
	}

	return scale.Attestation{
		AggregationBits: aggregationBits,
		Data:            data,
		Signature:       signature,
	}, nil
}

func (d VoluntaryExitResponse) ToScale() (scale.VoluntaryExit, error) {
	epoch, err := toUint64(d.Epoch)
	if err != nil {
		return scale.VoluntaryExit{}, err
	}

	validaterIndex, err := toUint64(d.ValidatorIndex)
	if err != nil {
		return scale.VoluntaryExit{}, err
	}

	return scale.VoluntaryExit{
		Epoch:          types.NewU64(epoch),
		ValidaterIndex: types.NewU64(validaterIndex),
	}, nil
}

func (d DepositResponse) ToScale() (scale.Deposit, error) {
	proofs := []types.H256{}

	for _, proofData := range d.Proof {
		proofs = append(proofs, types.NewH256(common.HexToHash(proofData).Bytes()))
	}

	amount, err := toUint64(d.Data.Amount)
	if err != nil {
		return scale.Deposit{}, err
	}

	pubkey, err := hexStringToByteArray(d.Data.Pubkey)
	if err != nil {
		return scale.Deposit{}, err
	}

	signature, err := hexStringToByteArray(d.Data.Signature)
	if err != nil {
		return scale.Deposit{}, err
	}

	return scale.Deposit{
		Proof: proofs,
		Data: scale.DepositData{
			Pubkey:                pubkey,
			WithdrawalCredentials: types.NewH256(common.HexToHash(d.Data.WithdrawalCredentials).Bytes()),
			Amount:                types.NewU64(amount),
			Signature:             signature,
		},
	}, nil
}

func (s SignedHeaderResponse) ToScale() (scale.SignedHeader, error) {
	message, err := s.Message.ToScale()
	if err != nil {
		return scale.SignedHeader{}, err
	}

	return scale.SignedHeader{
		Message:   message,
		Signature: s.Signature,
	}, nil
}

func (i IndexedAttestationResponse) ToScale() (scale.IndexedAttestation, error) {
	data, err := i.Data.ToScale()
	if err != nil {
		return scale.IndexedAttestation{}, err
	}

	attestationIndexes := []types.U64{}

	for _, index := range i.AttestingIndices {
		indexInt, err := toUint64(index)
		if err != nil {
			return scale.IndexedAttestation{}, err
		}

		attestationIndexes = append(attestationIndexes, types.NewU64(indexInt))
	}

	signature, err := hexStringToByteArray(i.Signature)
	if err != nil {
		return scale.IndexedAttestation{}, err
	}

	return scale.IndexedAttestation{
		AttestingIndices: attestationIndexes,
		Data:             data,
		Signature:        signature,
	}, nil
}

func (a AttestationDataResponse) ToScale() (scale.AttestationData, error) {
	slot, err := toUint64(a.Slot)
	if err != nil {
		return scale.AttestationData{}, err
	}

	index, err := toUint64(a.Index)
	if err != nil {
		return scale.AttestationData{}, err
	}

	source, err := a.Source.ToScale()
	if err != nil {
		return scale.AttestationData{}, err
	}

	target, err := a.Target.ToScale()
	if err != nil {
		return scale.AttestationData{}, err
	}

	return scale.AttestationData{
		Slot:            types.NewU64(slot),
		Index:           types.NewU64(index),
		BeaconBlockRoot: types.NewH256(common.HexToHash(a.BeaconBlockRoot).Bytes()),
		Source:          source,
		Target:          target,
	}, nil
}

func (c CheckpointResponse) ToScale() (scale.Checkpoint, error) {
	epoch, err := toUint64(c.Epoch)
	if err != nil {
		return scale.Checkpoint{}, err
	}

	return scale.Checkpoint{
		Epoch: types.NewU64(epoch),
		Root:  types.NewH256(common.HexToHash(c.Root).Bytes()),
	}, nil
}

func (p SyncCommitteePeriodPayload) ToJSON() SyncCommitteePeriodPayloadJSON {
	pubkeys := []string{}
	for _, pubkeyScale := range p.NextSyncCommittee.Pubkeys {
		pubkeys = append(pubkeys, bytesToHexString(pubkeyScale[:]))
	}

	nextSyncCommitteeBranch := []common.Hash{}
	for _, branch := range p.NextSyncCommitteeBranch {
		nextSyncCommitteeBranch = append(nextSyncCommitteeBranch, common.HexToHash(branch.Hex()))
	}

	finalityBranch := []common.Hash{}
	for _, branch := range p.FinalityBranch {
		finalityBranch = append(finalityBranch, common.HexToHash(branch.Hex()))
	}

	blockRootProof := []common.Hash{}
	for _, proof := range p.BlockRootProof {
		blockRootProof = append(blockRootProof, common.HexToHash(proof.Hex()))
	}

	return SyncCommitteePeriodPayloadJSON{
		AttestedHeader: BeaconHeader{
			Slot:          uint64(p.AttestedHeader.Slot),
			ProposerIndex: uint64(p.AttestedHeader.ProposerIndex),
			ParentRoot:    common.HexToHash(p.AttestedHeader.ParentRoot.Hex()),
			StateRoot:     common.HexToHash(p.AttestedHeader.StateRoot.Hex()),
			BodyRoot:      common.HexToHash(p.AttestedHeader.BodyRoot.Hex()),
		},
		NextSyncCommittee: SyncCommitteeResponse{
			Pubkeys:         pubkeys,
			AggregatePubkey: bytesToHexString(p.NextSyncCommittee.AggregatePubkey[:]),
		},
		NextSyncCommitteeBranch: nextSyncCommitteeBranch,
		FinalizedHeader: BeaconHeader{
			Slot:          uint64(p.FinalizedHeader.Slot),
			ProposerIndex: uint64(p.FinalizedHeader.ProposerIndex),
			ParentRoot:    common.HexToHash(p.FinalizedHeader.ParentRoot.Hex()),
			StateRoot:     common.HexToHash(p.FinalizedHeader.StateRoot.Hex()),
			BodyRoot:      common.HexToHash(p.FinalizedHeader.BodyRoot.Hex()),
		},
		FinalityBranch: finalityBranch,
		SyncAggregate: SyncAggregateResponse{
			SyncCommitteeBits:      bytesToHexString(p.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(p.SyncAggregate.SyncCommitteeSignature),
		},
		SyncCommitteePeriod: uint64(p.SyncCommitteePeriod),
		SignatureSlot:       uint64(p.SignatureSlot),
		BlockRootsHash:      common.HexToHash(p.BlockRootsHash.Hex()),
		BlockRootProof:      blockRootProof,
	}
}

func (p FinalizedHeaderPayload) ToJSON() FinalizedHeaderPayloadJSON {
	fmt.Println(p.AttestedHeader.ProposerIndex)
	fmt.Println(p.FinalizedHeader.ProposerIndex)

	finalityBranch := []common.Hash{}
	for _, branch := range p.FinalityBranch {
		finalityBranch = append(finalityBranch, common.HexToHash(branch.Hex()))
	}

	blockRootProof := []common.Hash{}
	for _, proof := range p.BlockRootProof {
		blockRootProof = append(blockRootProof, common.HexToHash(proof.Hex()))
	}

	return FinalizedHeaderPayloadJSON{
		AttestedHeader: BeaconHeader{
			Slot:          uint64(p.AttestedHeader.Slot),
			ProposerIndex: uint64(p.AttestedHeader.ProposerIndex),
			ParentRoot:    common.HexToHash(p.AttestedHeader.ParentRoot.Hex()),
			StateRoot:     common.HexToHash(p.AttestedHeader.StateRoot.Hex()),
			BodyRoot:      common.HexToHash(p.AttestedHeader.BodyRoot.Hex()),
		},
		FinalizedHeader: BeaconHeader{
			Slot:          uint64(p.FinalizedHeader.Slot),
			ProposerIndex: uint64(p.FinalizedHeader.ProposerIndex),
			ParentRoot:    common.HexToHash(p.FinalizedHeader.ParentRoot.Hex()),
			StateRoot:     common.HexToHash(p.FinalizedHeader.StateRoot.Hex()),
			BodyRoot:      common.HexToHash(p.FinalizedHeader.BodyRoot.Hex()),
		},
		FinalityBranch: finalityBranch,
		SyncAggregate: SyncAggregateResponse{
			SyncCommitteeBits:      bytesToHexString(p.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(p.SyncAggregate.SyncCommitteeSignature),
		},
		SignatureSlot:  uint64(p.SignatureSlot),
		BlockRootsHash: common.HexToHash(p.BlockRootsHash.Hex()),
		BlockRootProof: blockRootProof,
	}
}

func (h HeaderUpdate) ToJSON() HeaderUpdatePayloadJSON {
	proposerSlashings := []ProposerSlashingJSON{}
	for _, proposerSlashing := range h.Block.Body.ProposerSlashings {
		proposerSlashings = append(proposerSlashings, ProposerSlashingJSON{
			SignedHeader1: SignedHeaderJSON{
				Message: BeaconHeader{
					Slot:          uint64(proposerSlashing.SignedHeader1.Message.Slot),
					ProposerIndex: uint64(proposerSlashing.SignedHeader1.Message.ProposerIndex),
					ParentRoot:    common.HexToHash(proposerSlashing.SignedHeader1.Message.ParentRoot.Hex()),
					StateRoot:     common.HexToHash(proposerSlashing.SignedHeader1.Message.StateRoot.Hex()),
					BodyRoot:      common.HexToHash(proposerSlashing.SignedHeader1.Message.BodyRoot.Hex()),
				},
				Signature: bytesToHexString(proposerSlashing.SignedHeader1.Signature),
			},
			SignedHeader2: SignedHeaderJSON{
				Message: BeaconHeader{
					Slot:          uint64(proposerSlashing.SignedHeader2.Message.Slot),
					ProposerIndex: uint64(proposerSlashing.SignedHeader2.Message.ProposerIndex),
					ParentRoot:    common.HexToHash(proposerSlashing.SignedHeader2.Message.ParentRoot.Hex()),
					StateRoot:     common.HexToHash(proposerSlashing.SignedHeader2.Message.StateRoot.Hex()),
					BodyRoot:      common.HexToHash(proposerSlashing.SignedHeader2.Message.BodyRoot.Hex()),
				},
				Signature: bytesToHexString(proposerSlashing.SignedHeader2.Signature),
			},
		})
	}

	attesterSlashings := []AttesterSlashingJSON{}
	for _, attesterSlashing := range h.Block.Body.AttesterSlashings {
		attesterSlashings = append(attesterSlashings, AttesterSlashingJSON{
			Attestation1: IndexedAttestationJSON{
				AttestingIndices: toUint64Array(attesterSlashing.Attestation1.AttestingIndices),
				Data: AttestationDataJSON{
					Slot:            uint64(attesterSlashing.Attestation1.Data.Slot),
					Index:           uint64(attesterSlashing.Attestation1.Data.Index),
					BeaconBlockRoot: attesterSlashing.Attestation1.Data.BeaconBlockRoot.Hex(),
					Source: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation1.Data.Source.Epoch),
						Root:  attesterSlashing.Attestation1.Data.Source.Root.Hex(),
					},
					Target: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation1.Data.Target.Epoch),
						Root:  attesterSlashing.Attestation1.Data.Target.Root.Hex(),
					},
				},
				Signature: bytesToHexString(attesterSlashing.Attestation1.Signature),
			},
			Attestation2: IndexedAttestationJSON{
				AttestingIndices: toUint64Array(attesterSlashing.Attestation2.AttestingIndices),
				Data: AttestationDataJSON{
					Slot:            uint64(attesterSlashing.Attestation2.Data.Slot),
					Index:           uint64(attesterSlashing.Attestation2.Data.Index),
					BeaconBlockRoot: attesterSlashing.Attestation2.Data.BeaconBlockRoot.Hex(),
					Source: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation2.Data.Source.Epoch),
						Root:  attesterSlashing.Attestation2.Data.Source.Root.Hex(),
					},
					Target: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation2.Data.Target.Epoch),
						Root:  attesterSlashing.Attestation2.Data.Target.Root.Hex(),
					},
				},
				Signature: bytesToHexString(attesterSlashing.Attestation2.Signature),
			},
		})
	}

	attestations := []AttestationJSON{}
	for _, attestation := range h.Block.Body.Attestations {
		attestations = append(attestations, AttestationJSON{
			AggregationBits: bytesToHexString(attestation.AggregationBits),
			Data: AttestationDataJSON{
				Slot:            uint64(attestation.Data.Slot),
				Index:           uint64(attestation.Data.Index),
				BeaconBlockRoot: attestation.Data.BeaconBlockRoot.Hex(),
				Source: CheckpointJSON{
					Epoch: uint64(attestation.Data.Source.Epoch),
					Root:  attestation.Data.Source.Root.Hex(),
				},
				Target: CheckpointJSON{
					Epoch: uint64(attestation.Data.Target.Epoch),
					Root:  attestation.Data.Target.Root.Hex(),
				},
			},
			Signature: bytesToHexString(attestation.Signature),
		})
	}

	deposits := []DepositJSON{}
	for _, deposit := range h.Block.Body.Deposits {
		deposits = append(deposits, DepositJSON{
			Proof: scaleBranchToHex(deposit.Proof),
			Data: DepositDataJSON{
				Pubkey:                bytesToHexString(deposit.Data.Pubkey),
				WithdrawalCredentials: deposit.Data.WithdrawalCredentials.Hex(),
				Amount:                uint64(deposit.Data.Amount),
				Signature:             bytesToHexString(deposit.Data.Signature),
			},
		})
	}

	voluntaryExits := []VoluntaryExitJSON{}
	for _, voluntaryExit := range h.Block.Body.VoluntaryExits {
		voluntaryExits = append(voluntaryExits, VoluntaryExitJSON{
			Epoch:          uint64(voluntaryExit.Epoch),
			ValidatorIndex: uint64(voluntaryExit.ValidaterIndex),
		})
	}

	return HeaderUpdatePayloadJSON{
		Block: BlockJSON{
			Slot:          uint64(h.Block.Slot),
			ProposerIndex: uint64(h.Block.ProposerIndex),
			ParentRoot:    h.Block.ParentRoot.Hex(),
			StateRoot:     h.Block.StateRoot.Hex(),
			Body: BlockBodyJSON{
				RandaoReveal: bytesToHexString(h.Block.Body.RandaoReveal),
				Eth1Data: Eth1DataJSON{
					DepositRoot:  h.Block.Body.Eth1Data.DepositRoot.Hex(),
					DepositCount: uint64(h.Block.Body.Eth1Data.DepositCount),
					BlockHash:    h.Block.Body.Eth1Data.BlockHash.Hex(),
				},
				Graffiti:          h.Block.Body.Graffiti.Hex(),
				ProposerSlashings: proposerSlashings,
				AttesterSlashings: attesterSlashings,
				Attestations:      attestations,
				Deposits:          deposits,
				VoluntaryExits:    voluntaryExits,
				SyncAggregate: SyncAggregateResponse{
					SyncCommitteeBits:      bytesToHexString(h.Block.Body.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: bytesToHexString(h.Block.Body.SyncAggregate.SyncCommitteeSignature),
				},
				ExecutionPayload: ExecutionPayloadJSON{
					ParentHash:      h.Block.Body.ExecutionPayload.ParentHash.Hex(),
					FeeRecipient:    bytesToHexString(h.Block.Body.ExecutionPayload.FeeRecipient),
					StateRoot:       h.Block.Body.ExecutionPayload.StateRoot.Hex(),
					ReceiptsRoot:    h.Block.Body.ExecutionPayload.ReceiptsRoot.Hex(),
					LogsBloom:       bytesToHexString(h.Block.Body.ExecutionPayload.LogsBloom),
					PrevRandao:      h.Block.Body.ExecutionPayload.PrevRandao.Hex(),
					BlockNumber:     uint64(h.Block.Body.ExecutionPayload.BlockNumber),
					GasLimit:        uint64(h.Block.Body.ExecutionPayload.GasLimit),
					GasUsed:         uint64(h.Block.Body.ExecutionPayload.GasUsed),
					Timestamp:       uint64(h.Block.Body.ExecutionPayload.Timestamp),
					ExtraData:       bytesToHexString(h.Block.Body.ExecutionPayload.ExtraData),
					BaseFeePerGas:   h.Block.Body.ExecutionPayload.BaseFeePerGas.Uint64(),
					BlockHash:       h.Block.Body.ExecutionPayload.BlockHash.Hex(),
					TransactionRoot: h.Block.Body.ExecutionPayload.TransactionsRoot.Hex(),
				},
			},
		},
		SyncAggregate: SyncAggregateResponse{
			SyncCommitteeBits:      bytesToHexString(h.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(h.SyncAggregate.SyncCommitteeSignature),
		},
		SignatureSlot:                 uint64(h.SignatureSlot),
		BlockRootProof:                scaleBranchToHex(h.BlockRootProof),
		BlockRootProofFinalizedHeader: common.HexToHash(h.BlockRootProofFinalizedHeader.Hex()),
	}
}

func scaleBranchToHex(proofs []types.H256) []common.Hash {
	branch := []common.Hash{}

	for _, proof := range proofs {
		branch = append(branch, common.HexToHash(proof.Hex()))
	}

	return branch
}

func proofBranchToScale(proofs []common.Hash) []types.H256 {
	branch := []types.H256{}

	for _, proof := range proofs {
		branch = append(branch, types.NewH256(proof.Bytes()))
	}

	return branch
}

func toUint64(stringVal string) (uint64, error) {
	intVal, err := strconv.ParseUint(stringVal, 10, 64)
	if err != nil {
		return 0, err
	}

	return intVal, err
}

func toUint64Array(items []types.U64) []uint64 {
	result := []uint64{}

	for _, item := range items {
		result = append(result, uint64(item))
	}

	return result
}
