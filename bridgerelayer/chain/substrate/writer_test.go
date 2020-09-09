package substrate_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/stretchr/testify/assert"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/substrate"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/sr25519"
)

func TestWrite(t *testing.T) {
	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "Substrate")

	conn := substrate.NewConnection("ws://127.0.0.1:9944/", sr25519.Alice().AsKeyringPair(), log)

	messages := make(chan chain.Message, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer, err := substrate.NewWriter(conn, messages, log)
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

	// its OK to send a dummy message since the writer shouldn't care
	// whether the parachain processes the message or discards it.
	msg := chain.Message{AppID: chain.EthAppID, Payload: []byte{0,1,2}}

	err = writer.Write(ctx, &msg)
	if err != nil {
		t.Fatal(err)
	}

	assert.Equal(t, logrus.InfoLevel, hook.LastEntry().Level)
	assert.Equal(t, "Submitted message to Substrate", hook.LastEntry().Message)

}
