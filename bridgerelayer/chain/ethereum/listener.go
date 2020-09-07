package ethereum

import (
	"context"

	"golang.org/x/sync/errgroup"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
)

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn     *Connection
	apps     []Application
	messages chan<- core.Message
}

// NewListener initializes a new instance of Listener
func NewListener(conn *Connection, messages chan<- core.Message) (*Listener, error) {
	apps := LoadApplications(viper.GetString("ethereum.registry-path"))

	return &Listener{
		conn:     conn,
		apps:     apps,
		messages: messages,
	}, nil
}

func (li *Listener) Start(cxt context.Context, eg *errgroup.Group) error {

	eg.Go(func() error {
		return li.pollEvents(cxt)
	})

	return nil
}

func (li *Listener) pollEvents(ctx context.Context) error {
	log.Info("Polling started")
	events := make(chan etypes.Log)
	for _, app := range li.apps {
		query := makeQuery(app)
		_, err := li.conn.client.SubscribeFilterLogs(context.Background(), query, events)
		if err != nil {
			log.WithFields(log.Fields{
				"address": app.ID,
			}).Error("Failed to subscribe to application events")
		} else {
			log.WithFields(log.Fields{
				"address": app.ID,
			}).Info("Subscribed to application events")
		}
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case event := <-events:
			log.WithFields(
				log.Fields{
					"address":     event.Address.Hex(),
					"txHash":      event.TxHash.Hex(),
					"blockNumber": event.BlockNumber,
				},
			).Info("Witnessed transaction for application")

			msg, err := MakeMessageFromEvent(event, li.conn.kp)
			if err != nil {
				log.WithFields(log.Fields{
					"address":     event.Address.Hex(),
					"txHash":      event.TxHash.Hex(),
					"blockNumber": event.BlockNumber,
				}).Error("Failed to generate message from ethereum event")
			} else {
				li.messages <- *msg
			}
		}
	}
}

func makeQuery(app Application) ethereum.FilterQuery {
	address := common.HexToAddress(app.ID)
	signature := app.ABI.Events[EventName].ID.Hex()
	topic := common.HexToHash(signature)

	return ethereum.FilterQuery{
		Addresses: []common.Address{address},
		Topics:    [][]common.Hash{{topic}},
	}
}
