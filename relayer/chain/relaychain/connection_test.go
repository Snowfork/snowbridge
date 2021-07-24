// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
)

func TestConnect(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := relaychain.NewConnection("ws://127.0.0.1:9944/", log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}
