// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum_test

import (
	"context"
	"testing"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
)

func TestConnect(t *testing.T) {
	conn := ethereum.NewConnection("ws://localhost:8546", secp256k1.Alice())
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
}
