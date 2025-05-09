// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain_test

import (
	"context"
	"testing"

	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
)

func TestConnect(t *testing.T) {
	t.Skip("skip testing utility test")

	conn := parachain.NewConnection("ws://127.0.0.1:11144/", secp256k1.Alice().AsKeyringPair())
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}
