// Copyright 2026 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package xcmv5

import (
	"encoding/hex"
	"math/big"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Fixtures from staging-xcm 17 VersionedXcm::encode (see /tmp/xcmencode).
var (
	fixtureSimple = mustHex(
		"05080d00040201090704000b00f0ab75a40d0001010001010101010101010101010101010101010101010101010101010101010101010d00040201090704009d0f000101000202020202020202020202020202020202020202020202020202020202020202",
	)
	fixtureWithRefund = mustHex(
		"0510140d00040201090704000b00f0ab75a40d0001010001010101010101010101010101010101010101010101010101010101010101010d0102080001010001010101010101010101010101010101010101010101010101010101010101012c0909090909090909090909090909090909090909090909090909090909090909",
	)
	fixtureNonEther = mustHex(
		"05040d0004010000d107000101000101010101010101010101010101010101010101010101010101010101010101",
	)
)

func mustHex(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return b
}

func TestExtractServiceFeeSumsConfiguredRecipients(t *testing.T) {
	recipient := [32]byte{}
	for i := range recipient {
		recipient[i] = 1
	}
	other := [32]byte{}
	for i := range other {
		other[i] = 2
	}

	got := ExtractServiceFee(fixtureSimple, [][32]byte{recipient})
	assert.Equal(t, big.NewInt(15_000_000_000_000), got)

	got = ExtractServiceFee(fixtureSimple, [][32]byte{other})
	assert.Equal(t, big.NewInt(999), got)

	got = ExtractServiceFee(fixtureSimple, nil)
	assert.Equal(t, int64(0), got.Int64())

	got = ExtractServiceFee(fixtureSimple, [][32]byte{{3}})
	assert.Equal(t, int64(0), got.Int64())
}

func TestExtractServiceFeeSkipsRefundAndWild(t *testing.T) {
	recipient := [32]byte{}
	for i := range recipient {
		recipient[i] = 1
	}
	got := ExtractServiceFee(fixtureWithRefund, [][32]byte{recipient})
	assert.Equal(t, big.NewInt(15_000_000_000_000), got)
}

func TestExtractServiceFeeIgnoresNonEther(t *testing.T) {
	recipient := [32]byte{}
	for i := range recipient {
		recipient[i] = 1
	}
	got := ExtractServiceFee(fixtureNonEther, [][32]byte{recipient})
	assert.Equal(t, int64(0), got.Int64())
}

func TestParseRecipients(t *testing.T) {
	ids, err := ParseRecipients([]string{
		"0x2a975dad1c0fb19221a9288a0ae4b63ed29f3c625a73578117dde65482b8fc23",
		"5690961c19ef1f8519c7364d0c5290d0ad272cca827a11089f3248509710a500",
		"",
	})
	require.NoError(t, err)
	require.Len(t, ids, 2)
	assert.Equal(t, byte(0x2a), ids[0][0])
	assert.Equal(t, byte(0x56), ids[1][0])
}
