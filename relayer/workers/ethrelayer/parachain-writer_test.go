// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethrelayer_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus/hooks/test"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/ethrelayer"
)

func TestWrite(t *testing.T) {
	logger, _ := test.NewNullLogger()
	log := logger.WithField("chain", "Parachain")

	conn := parachain.NewConnection("ws://127.0.0.1:11144/", sr25519.Alice().AsKeyringPair(), log)

	payloads := make(chan ethrelayer.ParachainPayload, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer := ethrelayer.NewParachainWriter(conn, payloads, log)

	err := conn.Connect(ctx)
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
	header := chain.Header{
		HeaderData: "headerdata",
		ProofData:  "proofdata",
	}
	payload := ethrelayer.ParachainPayload{
		Header:   &header,
		Messages: []*chain.EthereumOutboundMessage{&message},
	}

	err = writer.WritePayload(ctx, &payload)
	if err != nil {
		t.Fatal(err)
	}
}
