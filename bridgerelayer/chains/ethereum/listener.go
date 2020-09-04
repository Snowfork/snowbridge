package ethereum

import (
	"context"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn *Connection
	apps []Application
	stop chan<- int
}

// NewListener initializes a new instance of Listener
func NewListener(conn Connection, stop chan<- int) (*Listener, error) {
	apps := LoadApplications(viper.GetString("ethereum.registry-path"))

	return &Listener{
		conn,
		apps,
		stop,
	}, nil
}

// Start initializes filtered subscriptions to each registered application
func (li Listener) Start() {
	appEvents := make(chan ctypes.Log)
	for _, app := range apps {
		query := li.buildQuery(app)

		// Start the contract subscription
		_, err := conn.client.SubscribeFilterLogs(context.Background(), query, appEvents)
		if err != nil {
			log.WithFields(
				log.Fields{
					"appID": app.ID,
				},
			).Error("Failed to subscribe to application events")
		} else {
			log.WithFields(
				log.Fields{
					"appID": app.ID,
				},
			).Info("Subscribed to application events")
		}
	}

	for {
		select {
		case log := <-appEvents:
			log.WithFields(
				log.Fields{
					"contractAddress": log.Address.Hex(),
					"txHash":          log.TxHash.Hex(),
					"blockNumber":     log.BlockNumber,
				},
			).Info("Witnessed transaction for application contract")
			eventData := types.NewEventData(log.Address, log)
		}
	}
}

func (li Listener) buildQuery(app types.Application) ethereum.FilterQuery {
	contractAddress := common.HexToAddress(app.ID)
	appEventSignature := app.ABI.Events[types.EventName].ID.Hex()
	appEventTopic := common.HexToHash(appEventSignature)

	return ethereum.FilterQuery{
		Addresses: []common.Address{contractAddress},
		Topics:    [][]common.Hash{{appEventTopic}},
	}
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
