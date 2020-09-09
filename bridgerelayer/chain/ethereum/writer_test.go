package ethereum_test

import (
	"context"
	"os"
	"testing"

	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/stretchr/testify/assert"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

func getKeypairFromEnv() *secp256k1.Keypair {
	priv, ok := os.LookupEnv("ARTEMIS_RELAY_ETHEREUM_KEY")
	if !ok || priv == "" {
		return nil
	}

	kp, err := secp256k1.NewKeypairFromString(priv)
	if err != nil {
		return nil
	}

	return kp
}

func TestWrite(t *testing.T) {
	kp := getKeypairFromEnv()
	if kp == nil {
		t.Skip("skipping test as ARTEMIS_RELAY_ETHEREUM_KEY not set")
	}

	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "Ethereum")

	conn := ethereum.NewConnection("ws://localhost:9545", kp, log)

	messages := make(chan chain.Message, 1)
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	writer, err := ethereum.NewWriter(conn, messages, log)
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
	msg := chain.Message{AppID: chain.EthAppID, Payload: []byte{0, 1, 2}}

	err = writer.Write(ctx, &msg)
	if err != nil {
		t.Fatal(err)
	}

	assert.Equal(t, logrus.InfoLevel, hook.LastEntry().Level)
	assert.Equal(t, "Submitted message to Substrate", hook.LastEntry().Message)

}
