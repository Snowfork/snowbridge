// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
)

// EventName is a common event name required by each application
const EventName = "AppEvent"

// EventData contains raw unencoded transaction data
type EventData struct {
	Contract common.Address
	Data     ctypes.Log
}

// NewEventData initializes a new instance of EventData
func NewEventData(contract common.Address, data ctypes.Log) EventData {
	return EventData{
		Contract: contract,
		Data:     data,
	}
}
