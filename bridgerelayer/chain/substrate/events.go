package substrate

import (
	"github.com/snowfork/go-substrate-rpc-client/types"
)

type EventSchedulerScheduled struct {
	Phase		types.Phase
	BlockNumber types.BlockNumber
	Index       types.U32
	Topics    	[]types.Hash
}

type EventSchedulerDispatched struct {
	Phase		types.Phase
	BlockNumber types.BlockNumber
	Index       types.U32
	MaybeID     types.OptionBytes8
	Result      types.DispatchResult
	Topics    	[]types.Hash
}

type EventBridgeReceived struct {
	Phase		types.Phase
	AccountID  	types.AccountID
	AppID       [32]byte
	Hash        types.H256
	Topics    	[]types.Hash
}

type EventAssetBurned struct {
	Phase		types.Phase
	AssetID     types.H160
	AccountID  	types.AccountID
	Amount      types.U256
	Topics    	[]types.Hash
}

type EventAssetMinted struct {
	Phase		types.Phase
	AssetID     types.H160
	AccountID  	types.AccountID
	Amount      types.U256
	Topics    	[]types.Hash
}

type EventAssetTransferred struct {
	Phase		types.Phase
	AssetID     types.H160
	Sender  	types.AccountID
	Receiver  	types.AccountID
	Amount      types.U256
	Topics    	[]types.Hash
}

type EventErc20Transfer struct {
	Phase		types.Phase
	TokenID    	types.H160
	AccountID  	types.AccountID
	Recipient   types.H160
	Amount		types.U256
	Topics    	[]types.Hash
}

type EventEthTransfer struct {
	Phase		types.Phase
	AccountID  	types.AccountID
	Recipient   types.H160
	Amount		types.U256
	Topics    	[]types.Hash
}

type Events struct {
	types.EventRecords
	Scheduler_Scheduled		[]EventSchedulerScheduled	//revive:disable-line
	Scheduler_Dispatched	[]EventSchedulerDispatched	//revive:disable-line
	Bridge_Received			[]EventBridgeReceived	//revive:disable-line
	Asset_Burned			[]EventAssetBurned		//revive:disable-line
	Asset_Minted			[]EventAssetMinted		//revive:disable-line
	Asset_Transferred		[]EventAssetTransferred	//revive:disable-line
	ETH_Transfer			[]EventEthTransfer		//revive:disable-line
	ERC20_Transfer			[]EventErc20Transfer	//revive:disable-line
}

