// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"github.com/snowfork/go-substrate-rpc-client/types"
)

type Events struct {
	types.EventRecords
	ETH_Transfer   []EventEthTransfer   //revive:disable-line
	ERC20_Transfer []EventErc20Transfer //revive:disable-line
}

type EventErc20Transfer struct {
	Phase     types.Phase
	TokenID   types.H160
	AccountID types.AccountID
	Recipient types.H160
	Amount    types.U256
	Topics    []types.Hash
}

type EventEthTransfer struct {
	Phase     types.Phase
	AccountID types.AccountID
	Recipient types.H160
	Amount    types.U256
	Topics    []types.Hash
}
