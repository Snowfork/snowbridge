// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/stretchr/testify/assert"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
)

var AppID = [20]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

func TestWrite(t *testing.T) {
	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "Substrate")

	conn := substrate.NewConnection("ws://127.0.0.1:9944/", sr25519.Alice().AsKeyringPair(), log)

	messages := make(chan []chain.Message, 1)
	headers := make(chan chain.Header, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer, err := substrate.NewWriter(conn, messages, headers, log)
	if err != nil {
		t.Fatal(err)
	}

	err = conn.Connect(ctx)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()

	err = writer.Start(ctx, eg)
	if err != nil {
		t.Fatal(err)
	}

	message := ethereum.Message{
		Data: []byte{0, 1, 2},
		VerificationInput: ethereum.VerificationInput{
			IsBasic: true,
			AsBasic: ethereum.VerificationBasic{
				BlockNumber: 47,
				EventIndex:  uint32(2),
			},
		},
	}

	err = writer.WriteMessage(ctx, &chain.Message{AppID: AppID, Payload: message})
	if err != nil {
		t.Fatal(err)
	}

	assert.Equal(t, logrus.InfoLevel, hook.LastEntry().Level)
	assert.Equal(t, "Submitted message to Substrate", hook.LastEntry().Message)

}
