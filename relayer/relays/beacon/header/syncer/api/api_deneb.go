package api

import (
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	beaconjson "github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/util"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

func DenebExecutionPayloadToScale(e *state.ExecutionPayloadDeneb, activeSpec config.ActiveSpec) (scale.ExecutionPayloadHeaderDeneb, error) {
	var payloadHeader scale.ExecutionPayloadHeaderDeneb
	transactionsContainer := state.TransactionsRootContainer{}
	transactionsContainer.Transactions = e.Transactions

	transactionsRoot, err := transactionsContainer.HashTreeRoot()
	if err != nil {
		return payloadHeader, err
	}

	var withdrawalRoot types.H256

	if activeSpec == config.Minimal {
		withdrawalContainer := state.WithdrawalsRootContainerMinimal{}
		withdrawalContainer.Withdrawals = e.Withdrawals
		withdrawalRoot, err = withdrawalContainer.HashTreeRoot()
	} else {
		withdrawalContainer := state.WithdrawalsRootContainerMainnet{}
		withdrawalContainer.Withdrawals = e.Withdrawals
		withdrawalRoot, err = withdrawalContainer.HashTreeRoot()
	}
	if err != nil {
		return payloadHeader, err
	}

	baseFeePerGas := big.Int{}
	// Change BaseFeePerGas back from little-endian to big-endian
	baseFeePerGas.SetBytes(util.ChangeByteOrder(e.BaseFeePerGas[:]))

	return scale.ExecutionPayloadHeaderDeneb{
		ParentHash:       types.NewH256(e.ParentHash[:]),
		FeeRecipient:     e.FeeRecipient,
		StateRoot:        types.NewH256(e.StateRoot[:]),
		ReceiptsRoot:     types.NewH256(e.ReceiptsRoot[:]),
		LogsBloom:        e.LogsBloom[:],
		PrevRandao:       types.NewH256(e.PrevRandao[:]),
		BlockNumber:      types.NewU64(e.BlockNumber),
		GasLimit:         types.NewU64(e.GasLimit),
		GasUsed:          types.NewU64(e.GasUsed),
		Timestamp:        types.NewU64(e.Timestamp),
		ExtraData:        e.ExtraData,
		BaseFeePerGas:    types.NewU256(baseFeePerGas),
		BlockHash:        types.NewH256(e.BlockHash[:]),
		TransactionsRoot: transactionsRoot,
		WithdrawalsRoot:  withdrawalRoot,
		BlobGasUsed:      types.NewU64(e.BlobGasUsed),
		ExcessBlobGas:    types.NewU64(e.ExcessBlobGas),
	}, nil
}

func DenebJsonExecutionPayloadHeaderToScale(e *beaconjson.FullExecutionPayloadHeaderJson) (scale.ExecutionPayloadHeaderDeneb, error) {
	var executionPayloadHeader scale.ExecutionPayloadHeaderDeneb
	var baseFeePerGas big.Int
	baseFeePerGasU64, err := util.ToUint64(e.BaseFeePerGas)
	if err != nil {
		return executionPayloadHeader, err
	}
	blockNumber, err := util.ToUint64(e.BlockNumber)
	if err != nil {
		return executionPayloadHeader, err
	}
	baseFeePerGas.SetUint64(baseFeePerGasU64)
	gasLimit, err := util.ToUint64(e.GasLimit)
	if err != nil {
		return executionPayloadHeader, err
	}
	gasUsed, err := util.ToUint64(e.GasUsed)
	if err != nil {
		return executionPayloadHeader, err
	}
	timestamp, err := util.ToUint64(e.Timestamp)
	if err != nil {
		return executionPayloadHeader, err
	}
	blobGasUsed, _ := util.ToUint64(e.BlobGasUsed)
	excessBlobGas, _ := util.ToUint64(e.ExcessBlobGas)
	return scale.ExecutionPayloadHeaderDeneb{
		ParentHash:       types.NewH256(common.HexToHash(e.ParentHash).Bytes()),
		FeeRecipient:     types.NewH160(common.HexToAddress(e.FeeRecipient).Bytes()),
		StateRoot:        types.NewH256(common.HexToHash(e.StateRoot).Bytes()),
		ReceiptsRoot:     types.NewH256(common.HexToHash(e.ReceiptsRoot).Bytes()),
		LogsBloom:        common.FromHex(e.LogsBloom),
		PrevRandao:       types.NewH256(common.HexToHash(e.PrevRandao).Bytes()),
		BlockNumber:      types.NewU64(blockNumber),
		GasLimit:         types.NewU64(gasLimit),
		GasUsed:          types.NewU64(gasUsed),
		Timestamp:        types.NewU64(timestamp),
		ExtraData:        common.FromHex(e.ExtraData),
		BaseFeePerGas:    types.NewU256(baseFeePerGas),
		BlockHash:        types.NewH256(common.HexToHash(e.BlockHash).Bytes()),
		TransactionsRoot: types.NewH256(common.HexToHash(e.TransactionsRoot).Bytes()),
		WithdrawalsRoot:  types.NewH256(common.HexToHash(e.WithdrawalsRoot).Bytes()),
		BlobGasUsed:      types.NewU64(blobGasUsed),
		ExcessBlobGas:    types.NewU64(excessBlobGas),
	}, nil
}
