package substrate

import (
	"github.com/snowfork/go-substrate-rpc-client/types"
)

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
	Amount		types.U256
	Topics    	[]types.Hash
}

type EventEthTransfer struct {
	Phase		types.Phase
	AccountID  	types.AccountID
	Amount		types.U256
	Topics    	[]types.Hash
}

type Events struct {
	types.EventRecords
	Bridge_Received		[]EventBridgeReceived	//revive:disable-line
	Asset_Burned		[]EventAssetBurned		//revive:disable-line
	Asset_Minted		[]EventAssetMinted		//revive:disable-line
	Asset_Transferred	[]EventAssetTransferred	//revive:disable-line
	ETH_Transfer		[]EventEthTransfer		//revive:disable-line
	ERC20_Transfer		[]EventErc20Transfer	//revive:disable-line
}

