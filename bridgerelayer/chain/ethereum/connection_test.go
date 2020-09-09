// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

var TestEndpoint = "ws://localhost:9545"
var AliceKP = secp256k1.AliceKP()

func TestConnect(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := ethereum.NewConnection(TestEndpoint,  AliceKP, log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}
