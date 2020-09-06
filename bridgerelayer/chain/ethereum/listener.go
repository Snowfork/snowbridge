package ethereum

import (
	"context"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn    *Connection
	channel chain.Channel
	apps    []types.Application
	stop    <-chan int
}

// NewListener initializes a new instance of Listener
func NewListener(conn *Connection, stop <-chan int) (*Listener, error) {
	apps := LoadApplications(viper.GetString("ethereum.registry-path"))

	return &Listener{
		conn: conn,
		apps: apps,
		stop: stop,
	}, nil
}

func (li *Listener) Start() error {
	go func() {
		li.pollEvents()
	}()

	return nil
}

func (li *Listener) pollEvents() {
	log.Info("Polling started")
	events := make(chan ctypes.Log)
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
		case <-li.stop:
			log.Info("Polling stopped")
			return
		case event := <-events:
			log.WithFields(
				log.Fields{
					"address":     event.Address.Hex(),
					"txHash":      event.TxHash.Hex(),
					"blockNumber": event.BlockNumber,
				},
			).Info("Witnessed transaction for application")
		}
	}
}

func makeQuery(app types.Application) ethereum.FilterQuery {
	address := common.HexToAddress(app.ID)
	signature := app.ABI.Events[types.EventName].ID.Hex()
	topic := common.HexToHash(signature)

	return ethereum.FilterQuery{
		Addresses: []common.Address{address},
		Topics:    [][]common.Hash{{topic}},
	}
}

func (li *Listener) setChannel(ch chain.Channel) {
	li.channel = ch
}

// Route packages tx data as a packet and relays it to the bridge
// func (er Router) Route(eventData types.EventData) error {

// 	appAddress := eventData.Contract.Bytes()
// 	var appID [32]byte
// 	copy(appID[:], appAddress)

// 	packet, err := er.buildPacket(eventData.Contract, eventData.Data)
// 	if err != nil {
// 		return err
// 	}

// 	err = er.sendPacket(appID, packet)
// 	if err != nil {
// 		return err
// 	}

// 	return nil
// }
