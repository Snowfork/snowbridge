// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain_test

import (
	"testing"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate"
	"github.com/stretchr/testify/assert"
)

func TestMortalEra(t *testing.T) {
	era := substrate.NewMortalEra(1)
	assert.Equal(t, era.AsMortalEra.First, byte(21))
	assert.Equal(t, era.AsMortalEra.Second, byte(0))

	era = substrate.NewMortalEra(63)
	assert.Equal(t, era.AsMortalEra.First, byte(245))
	assert.Equal(t, era.AsMortalEra.Second, byte(3))

	era = substrate.NewMortalEra(64)
	assert.Equal(t, era.AsMortalEra.First, byte(5))
	assert.Equal(t, era.AsMortalEra.Second, byte(0))
}
