// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/stretchr/testify/assert"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

func TestWrite(t *testing.T) {
	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "Parachain")

	conn := parachain.NewConnection("ws://127.0.0.1:11144/", sr25519.Alice().AsKeyringPair(), log)

	messages := make(chan []chain.Message, 1)
	headers := make(chan chain.Header, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer, err := parachain.NewWriter(conn, messages, headers, log)
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

	var args []interface{}
	args = append(args,
		chainTypes.Message{
			Data: []byte{1, 2, 3},
			Proof: chainTypes.Proof{
				BlockHash: types.NewH256([]byte{1, 2, 3}),
				TxIndex:   1,
				Data:      chainTypes.NewProofData(),
			},
		},
	)

	message := chain.EthereumOutboundMessage{
		Call: "BasicInboundChannel.submit",
		Args: args,
	}

	err = writer.WriteMessages(ctx, []*chain.EthereumOutboundMessage{&message})
	if err != nil {
		t.Fatal(err)
	}

	assert.Equal(t, logrus.InfoLevel, hook.LastEntry().Level)
	assert.Equal(t, "Submitted messages to Substrate", hook.LastEntry().Message)

}
