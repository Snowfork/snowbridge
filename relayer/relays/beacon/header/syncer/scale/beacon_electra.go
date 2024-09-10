package scale

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type ExecutionPayloadHeaderElectra struct {
	ParentHash                types.H256
	FeeRecipient              types.H160
	StateRoot                 types.H256
	ReceiptsRoot              types.H256
	LogsBloom                 []byte
	PrevRandao                types.H256
	BlockNumber               types.U64
	GasLimit                  types.U64
	GasUsed                   types.U64
	Timestamp                 types.U64
	ExtraData                 []byte
	BaseFeePerGas             types.U256
	BlockHash                 types.H256
	TransactionsRoot          types.H256
	WithdrawalsRoot           types.H256
	BlobGasUsed               types.U64
	ExcessBlobGas             types.U64
	DepositRequestsRoot       types.H256
	WithdrawalRequestsRoot    types.H256
	ConsolidationRequestsRoot types.H256
}

type DepositRequest struct {
	Pubkey                []byte
	WithdrawalCredentials []byte
	Amount                types.U64
	Signature             []byte
	Index                 types.U64
}

type WithdrawalRequest struct {
	SourceAddress   []byte
	ValidatorPubkey []byte
	Amount          types.U64
}

type ConsolidationRequest struct {
	SourceAddress []byte
	SourcePubkey  []byte
	TargetPubkey  []byte
}

func (e *ExecutionPayloadHeaderElectra) ToJSON() json.ExecutionPayloadHeaderElectra {
	return json.ExecutionPayloadHeaderElectra{
		ParentHash:                e.ParentHash.Hex(),
		FeeRecipient:              util.BytesToHexString(e.FeeRecipient[:]),
		StateRoot:                 e.StateRoot.Hex(),
		ReceiptsRoot:              e.ReceiptsRoot.Hex(),
		LogsBloom:                 util.BytesToHexString(e.LogsBloom),
		PrevRandao:                e.PrevRandao.Hex(),
		BlockNumber:               uint64(e.BlockNumber),
		GasLimit:                  uint64(e.GasLimit),
		GasUsed:                   uint64(e.GasUsed),
		Timestamp:                 uint64(e.Timestamp),
		ExtraData:                 util.BytesToHexString(e.ExtraData),
		BaseFeePerGas:             e.BaseFeePerGas.Uint64(),
		BlockHash:                 e.BlockHash.Hex(),
		TransactionsRoot:          e.TransactionsRoot.Hex(),
		WithdrawalsRoot:           e.WithdrawalsRoot.Hex(),
		BlobGasUsed:               uint64(e.BlobGasUsed),
		ExcessBlobGas:             uint64(e.ExcessBlobGas),
		DepositRequestsRoot:       e.DepositRequestsRoot.Hex(),
		WithdrawalRequestsRoot:    e.WithdrawalRequestsRoot.Hex(),
		ConsolidationRequestsRoot: e.ConsolidationRequestsRoot.Hex(),
	}
}
