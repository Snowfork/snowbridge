package scale

import "github.com/snowfork/go-substrate-rpc-client/v4/types"

type ExecutionPayloadHeaderDeneb struct {
	ParentHash       types.H256
	FeeRecipient     types.H160
	StateRoot        types.H256
	ReceiptsRoot     types.H256
	LogsBloom        []byte
	PrevRandao       types.H256
	BlockNumber      types.U64
	GasLimit         types.U64
	GasUsed          types.U64
	Timestamp        types.U64
	ExtraData        []byte
	BaseFeePerGas    types.U256
	BlockHash        types.H256
	TransactionsRoot types.H256
	WithdrawalsRoot  types.H256
	BlobGasUsed      types.U64
	ExcessBlobGas    types.U64
}
