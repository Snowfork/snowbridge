// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum_test

import (
	"context"
	"testing"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	ethereumRelay "github.com/snowfork/snowbridge/relayer/relays/ethereum"
)

func TestWrite(t *testing.T) {
	conn := parachain.NewConnection("ws://127.0.0.1:11144/", sr25519.Alice().AsKeyringPair())

	payloads := make(chan ethereumRelay.ParachainPayload, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer := ethereumRelay.NewParachainWriter(conn, payloads)

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
		parachain.Message{
			Data: []byte{1, 2, 3},
			Proof: parachain.Proof{
				BlockHash: types.NewH256([]byte{1, 2, 3}),
				TxIndex:   1,
				Data:      parachain.NewProofData(),
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
	payload := ethereumRelay.ParachainPayload{
		Header:   &header,
		Messages: []*chain.EthereumOutboundMessage{&message},
	}

	err = writer.WritePayload(ctx, &payload)
	if err != nil {
		t.Fatal(err)
	}
}
