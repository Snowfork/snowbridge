package substrate

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/sr25519"
	"github.com/spf13/viper"
)

// Chain ...
type Chain struct {
	listener *Listener
	writer   *Writer
	conn     *Connection
}

const Name = "Substrate"

// NewChain ...
func NewChain(ethMessages chan core.Message, subMessages chan core.Message) (*Chain, error) {

	// Validate and load configuration
	keys := []string{
		"substrate.endpoint",
		"substrate.private-key",
		"substrate.block-retry-limit",
		"substrate.block-retry-interval",
	}
	for _, key := range keys {
		if !viper.IsSet(key) {
			return nil, fmt.Errorf("Config key %q not set", key)
		}
	}
	endpoint := viper.GetString("substrate.endpoint")
	secret := viper.GetString("substrate.private-key")
	blockRetryLimit := viper.GetUint("substrate.block-retry-limit")
	blockRetryInterval := viper.GetUint("substrate.block-retry-interval")

	// Generate keypair from secret
	kp, err := sr25519.NewKeypairFromSeed(secret, "")
	if err != nil {
		return nil, err
	}

	conn := NewConnection(endpoint, kp.AsKeyringPair())

	listener := NewListener(
		conn,
		subMessages,
		blockRetryLimit,
		blockRetryInterval,
	)

	writer, err := NewWriter(conn, ethMessages)
	if err != nil {
		return nil, err
	}

	return &Chain{
		conn:     conn,
		listener: listener,
		writer:   writer,
	}, nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group) error {

	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	err = ch.listener.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = ch.writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}

func (ch *Chain) Stop() {
	if ch.conn != nil {
		ch.conn.Close()
	}
}

func (ch *Chain) Name() string {
	return Name
}
