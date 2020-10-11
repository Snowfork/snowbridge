package substrate

import (
	"bytes"
	"fmt"
	"reflect"

	"github.com/ethereum/go-ethereum/log"
	"github.com/centrifuge/go-substrate-rpc-client/scale"
	"github.com/centrifuge/go-substrate-rpc-client/types"
)

type Event struct {
	ID     [2]uint8
	Name   [2]string
	Phase  types.Phase
	Topics []types.Hash
	Fields interface{}
}

// System

type SystemExtrinsicSuccess struct {
	DispatchInfo types.DispatchInfo
}

type SystemExtrinsicFailed struct {
	DispatchError types.DispatchError
	DispatchInfo  types.DispatchInfo
}

type SystemCodeUpdated struct{}

type SystemNewAccount struct {
	AccountID types.AccountID
}

type SystemKilledAccount struct {
	AccountID types.AccountID
}

// Grandpa

type GrandpaNewAuthorities struct {
	NewAuthorities []struct {
		AuthorityID     types.AuthorityID
		AuthorityWeight types.U64
	}
}

type GrandpaPaused struct{}

type GrandpaResumed struct{}

type BalancesEndowed struct {
	Who     types.AccountID
	Balance types.U128
}

// Balances

type BalancesDustLost struct {
	Who     types.AccountID
	Balance types.U128
}

type BalancesTransfer struct {
	From  types.AccountID
	To    types.AccountID
	Value types.U128
}

type BalancesBalanceSet struct {
	Who      types.AccountID
	Free     types.U128
	Reserved types.U128
}

type BalancesDeposit struct {
	Who     types.AccountID
	Balance types.U128
}

type BalancesReserved struct {
	Who     types.AccountID
	Balance types.U128
}

type BalancesUnreserved struct {
	Who     types.AccountID
	Balance types.U128
}

// Asset

type AssetBurned struct {
	AssetID   types.H160
	AccountID types.AccountID
	Amount    types.U256
}

type AssetMinted struct {
	AssetID   types.H160
	AccountID types.AccountID
	Amount    types.U256
}

type AssetTransferred struct {
	AssetID  types.H160
	Sender   types.AccountID
	Receiver types.AccountID
	Amount   types.U256
}

// ETH

type ETHTransfer struct {
	AccountID types.AccountID
	Recipient types.H160
	Amount    types.U256
}

// ERC20

type ERC20Transfer struct {
	TokenID   types.H160
	AccountID types.AccountID
	Recipient types.H160
	Amount    types.U256
}

type EventDecoderError struct {
}

type TypeMap map[[2]string]reflect.Type

type EventDecoder struct {
	meta  *types.Metadata
	Types TypeMap
}

func NewEventDecoder(meta *types.Metadata) *EventDecoder {

	tm := make(TypeMap)

	tm[[2]string{"System", "ExtrinsicSuccess"}] = reflect.TypeOf(SystemExtrinsicSuccess{})
	tm[[2]string{"System", "ExtrinsicFailed"}] = reflect.TypeOf(SystemExtrinsicFailed{})
	tm[[2]string{"System", "CodeUpdated"}] = reflect.TypeOf(SystemCodeUpdated{})
	tm[[2]string{"System", "NewAccount"}] = reflect.TypeOf(SystemNewAccount{})
	tm[[2]string{"System", "KilledAccount"}] = reflect.TypeOf(SystemKilledAccount{})
	tm[[2]string{"Grandpa", "NewAuthorities"}] = reflect.TypeOf(GrandpaNewAuthorities{})
	tm[[2]string{"Grandpa", "Paused"}] = reflect.TypeOf(GrandpaPaused{})
	tm[[2]string{"Grandpa", "Resumed"}] = reflect.TypeOf(GrandpaResumed{})
	tm[[2]string{"Balances", "Endowed"}] = reflect.TypeOf(BalancesEndowed{})
	tm[[2]string{"Balances", "DustLost"}] = reflect.TypeOf(BalancesDustLost{})
	tm[[2]string{"Balances", "Transfer"}] = reflect.TypeOf(BalancesTransfer{})
	tm[[2]string{"Balances", "BalanceSet"}] = reflect.TypeOf(BalancesBalanceSet{})
	tm[[2]string{"Balances", "Deposit"}] = reflect.TypeOf(BalancesDeposit{})
	tm[[2]string{"Balances", "Reserved"}] = reflect.TypeOf(BalancesReserved{})
	tm[[2]string{"Balances", "Unreserved"}] = reflect.TypeOf(BalancesUnreserved{})
	tm[[2]string{"Asset", "Burned"}] = reflect.TypeOf(AssetBurned{})
	tm[[2]string{"Asset", "Minted"}] = reflect.TypeOf(AssetMinted{})
	tm[[2]string{"Asset", "Transferred"}] = reflect.TypeOf(AssetTransferred{})
	tm[[2]string{"ETH", "Transfer"}] = reflect.TypeOf(ETHTransfer{})
	tm[[2]string{"ERC20", "Transfer"}] = reflect.TypeOf(ERC20Transfer{})

	return &EventDecoder{
		meta:  meta,
		Types: tm,
	}
}

func (ed *EventDecoder) Decode(records []byte) ([]Event, error) {

	decoder := scale.NewDecoder(bytes.NewReader(records))

	// determine number of events
	length, err := decoder.DecodeUintCompact()
	if err != nil {
		return nil, err
	}

	events := []Event{}

	// iterate over events
	for i := uint64(0); i < length.Uint64(); i++ {
		log.Trace(fmt.Sprintf("decoding event #%v", i))

		// Decode phase
		phase := types.Phase{}
		err := decoder.Decode(&phase)
		if err != nil {
			return nil, fmt.Errorf("unable to decode Phase for event #%v: %v", i, err)
		}

		// Decode event ID
		id := types.EventID{}
		err = decoder.Decode(&id)
		if err != nil {
			return nil, fmt.Errorf("unable to decode EventID for event #%v: %v", i, err)
		}

		// Ask metadata for method and event name
		moduleName, eventName, err := ed.meta.FindEventNamesForEventID(id)
		if err != nil {
			return nil, fmt.Errorf("unable to find event with EventID %v in metadata for event #%v: %s", id, i, err)
		}

		key := [2]string{string(moduleName), string(eventName)}
		holderType, ok := ed.Types[key]
		if !ok {
			return nil, fmt.Errorf("event #%v (%s.%s) is not decodable", i, moduleName, eventName)
		}

		holder := reflect.New(holderType)
		numFields := holder.Elem().NumField()

		// Decode event fields
		for j := 0; j < numFields; j++ {
			err = decoder.Decode(holder.Elem().FieldByIndex([]int{j}).Addr().Interface())
			if err != nil {
				return nil, fmt.Errorf(
					"unable to decode field %v of event #%v with EventID %v, field %v_%v: %v", j, i, id, moduleName,
					eventName, err,
				)
			}
		}

		// Decode topics
		topics := []types.Hash{}
		err = decoder.Decode(&topics)
		if err != nil {
			return nil, fmt.Errorf("unable to decode topics for event #%v: %v", i, err)
		}

		event := Event{
			ID:     [2]uint8{uint8(id[0]), uint8(id[1])},
			Name:   [2]string{string(moduleName), string(eventName)},
			Phase:  phase,
			Topics: topics,
			Fields: holder.Elem().Interface(),
		}

		events = append(events, event)

	}

	return events, nil
}
