package api

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

func ElectraExecutionPayloadToScale(e *state.ExecutionPayloadElectra) (scale.ExecutionPayloadHeaderElectra, error) {
	var payloadHeader scale.ExecutionPayloadHeaderElectra
	transactionsContainer := state.TransactionsRootContainer{}
	transactionsContainer.Transactions = e.Transactions

	transactionsRoot, err := transactionsContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	withdrawalContainer := state.WithdrawalsRootContainerMainnet{}
	withdrawalContainer.Withdrawals = e.Withdrawals
	withdrawalRoot, err := withdrawalContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	depositRequestsContainer := state.DepositRequestsContainer{}
	depositRequestsContainer.DepositRequests = e.DepositRequests
	depositRequestsRoot, err := depositRequestsContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	withdrawalRequestsContainer := state.WithdrawalRequestsContainer{}
	withdrawalRequestsContainer.WithdrawalRequests = e.WithdrawalRequests
	withdrawalRequestsRoot, err := withdrawalRequestsContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	consolidationRequestsContainer := state.ConsolidationRequestsContainer{}
	consolidationRequestsContainer.ConsolidationRequests = e.ConsolidationRequests
	consolidationRequestsRoot, err := consolidationRequestsContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	baseFeePerGas := big.Int{}
	// Change BaseFeePerGas back from little-endian to big-endian
	baseFeePerGas.SetBytes(util.ChangeByteOrder(e.BaseFeePerGas[:]))

	return scale.ExecutionPayloadHeaderElectra{
		ParentHash:                types.NewH256(e.ParentHash[:]),
		FeeRecipient:              e.FeeRecipient,
		StateRoot:                 types.NewH256(e.StateRoot[:]),
		ReceiptsRoot:              types.NewH256(e.ReceiptsRoot[:]),
		LogsBloom:                 e.LogsBloom[:],
		PrevRandao:                types.NewH256(e.PrevRandao[:]),
		BlockNumber:               types.NewU64(e.BlockNumber),
		GasLimit:                  types.NewU64(e.GasLimit),
		GasUsed:                   types.NewU64(e.GasUsed),
		Timestamp:                 types.NewU64(e.Timestamp),
		ExtraData:                 e.ExtraData,
		BaseFeePerGas:             types.NewU256(baseFeePerGas),
		BlockHash:                 types.NewH256(e.BlockHash[:]),
		TransactionsRoot:          transactionsRoot,
		WithdrawalsRoot:           withdrawalRoot,
		BlobGasUsed:               types.NewU64(e.BlobGasUsed),
		ExcessBlobGas:             types.NewU64(e.ExcessBlobGas),
		DepositRequestsRoot:       depositRequestsRoot,
		WithdrawalRequestsRoot:    withdrawalRequestsRoot,
		ConsolidationRequestsRoot: consolidationRequestsRoot,
	}, nil
}

func (a AttesterSlashingResponse) ToFastSSZElectra() (*state.AttesterSlashingElectra, error) {
	attestation1, err := a.Attestation1.ToFastSSZElectra()
	if err != nil {
		return nil, err
	}

	attestation2, err := a.Attestation2.ToFastSSZElectra()
	if err != nil {
		return nil, err
	}

	return &state.AttesterSlashingElectra{
		Attestation1: attestation1,
		Attestation2: attestation2,
	}, nil
}

func (i IndexedAttestationResponse) ToFastSSZElectra() (*state.IndexedAttestationElectra, error) {
	data, err := i.Data.ToFastSSZ()
	if err != nil {
		return nil, err
	}

	attestationIndexes := []uint64{}
	for _, index := range i.AttestingIndices {
		indexInt, err := util.ToUint64(index)
		if err != nil {
			return nil, err
		}

		attestationIndexes = append(attestationIndexes, indexInt)
	}

	signature, err := util.HexStringToByteArray(i.Signature)
	if err != nil {
		return nil, err
	}

	return &state.IndexedAttestationElectra{
		AttestationIndices: attestationIndexes,
		Data:               data,
		Signature:          signature,
	}, nil
}

func (a AttestationResponse) ToFastSSZElectra() (*state.AttestationElectra, error) {
	data, err := a.Data.ToFastSSZ()
	if err != nil {
		return nil, err
	}

	aggregationBits, err := util.HexStringToByteArray(a.AggregationBits)
	if err != nil {
		return nil, err
	}

	signature, err := util.HexStringTo96Bytes(a.Signature)
	if err != nil {
		return nil, err
	}

	committeeBits, err := util.HexStringToByteArray(a.CommitteeBits)
	if err != nil {
		return nil, err
	}

	return &state.AttestationElectra{
		AggregationBits: aggregationBits,
		Data:            data,
		Signature:       signature,
		CommitteeBits:   committeeBits,
	}, nil
}
