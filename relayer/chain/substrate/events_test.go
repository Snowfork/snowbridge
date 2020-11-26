package substrate

import (
	"math/big"
	"testing"

	"github.com/centrifuge/go-substrate-rpc-client/types"
	"github.com/stretchr/testify/assert"
)

func TestDecodeEvents(t *testing.T) {
	decoder := NewEventDecoder(MetadataExemplary)

	records := types.MustHexDecodeString(
		"0c0000000000000080e36a090000000002000000010000000202d43593c7" +
			"15fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d8eaf" +
			"04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48" +
			"00805f2bd9fbb40100000000000000000000010000000000c0769f0b0000" +
			"0000000000",
	)

	events, err := decoder.Decode(records)
	if err != nil {
		t.Error(err)
	}

	assert.Equal(t, events,
		[]Event{
			{
				ID:     [2]uint8{0x0, 0x0},
				Name:   [2]string{"System", "ExtrinsicSuccess"},
				Phase:  types.Phase{IsApplyExtrinsic: true, AsApplyExtrinsic: 0x0, IsFinalization: false},
				Topics: []types.Hash{},
				Fields: SystemExtrinsicSuccess{
					DispatchInfo: types.DispatchInfo{
						Weight:  0x96ae380,
						Class:   types.DispatchClass{IsNormal: false, IsOperational: false},
						PaysFee: false,
					},
				},
			},
			{
				ID:     [2]uint8{0x2, 0x2},
				Name:   [2]string{"Balances", "Transfer"},
				Phase:  types.Phase{IsApplyExtrinsic: true, AsApplyExtrinsic: 0x1, IsFinalization: false},
				Topics: []types.Hash{},
				Fields: BalancesTransfer{
					From: types.AccountID{0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x4, 0xa9, 0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d, 0xa2, 0x7d},
					To:   types.AccountID{0x8e, 0xaf, 0x4, 0x15, 0x16, 0x87, 0x73, 0x63, 0x26, 0xc9, 0xfe, 0xa1, 0x7e, 0x25, 0xfc, 0x52, 0x87, 0x61, 0x36, 0x93, 0xc9, 0x12, 0x90, 0x9c, 0xb2, 0x26, 0xaa, 0x47, 0x94, 0xf2, 0x6a, 0x48},
					Value: types.U128{
						Int: big.NewInt(0).SetBits([]big.Word{0x1b4fbd92b5f8000}),
					},
				},
			},
			{
				ID:     [2]uint8{0x0, 0x0},
				Name:   [2]string{"System", "ExtrinsicSuccess"},
				Phase:  types.Phase{IsApplyExtrinsic: true, AsApplyExtrinsic: 0x1, IsFinalization: false},
				Topics: []types.Hash{},
				Fields: SystemExtrinsicSuccess{
					DispatchInfo: types.DispatchInfo{
						Weight:  0xb9f76c0,
						Class:   types.DispatchClass{IsNormal: true, IsOperational: false},
						PaysFee: false,
					},
				},
			},
		},
	)
}
