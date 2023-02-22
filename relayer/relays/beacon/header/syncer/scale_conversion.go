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

func scaleBranchToHex(proofs []types.H256) []common.Hash {
	branch := []common.Hash{}

	for _, proof := range proofs {
		branch = append(branch, common.HexToHash(proof.Hex()))
	}

	return branch
}

func scaleBranchToString(proofs []types.H256) []string {
	branch := []string{}

	for _, proof := range proofs {
		branch = append(branch, proof.Hex())
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
