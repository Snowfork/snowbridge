package ethereum

import (
	"context"
	"fmt"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn         Connection
	RegistryPath string
	logs         chan<- types.EventData
	errs         chan<- error
}

// NewListener initializes a new instance of Listener
func NewListener(conn Connection, registryPath string) (*Listener, error) {
	return &Listener{
		WebsocketURL: websocketURL,
		RegistryPath: registryPath,
	}, nil
}

// Start initializes filtered subscriptions to each registered application
func (es Listener) Start(logs chan<- types.EventData, errs chan<- error) {
	apps := LoadApplications(es.RegistryPath)

	es.logs = logs
	es.errs = errs

	client, err := ethclient.Dial(es.WebsocketURL)
	if err != nil {
		es.errs <- err
	}

	chainID, err := client.NetworkID(context.Background())
	if err != nil {
		es.errs <- err
	}
	log.Info(fmt.Sprintf("Connected to Ethereum chain ID %s\n", chainID))

	// Start application subscriptions
	appEvents := make(chan ctypes.Log)
	for _, app := range apps {
		query := es.buildSubscriptionFilter(app)

		// Start the contract subscription
		_, err := client.SubscribeFilterLogs(context.Background(), query, appEvents)
		if err != nil {
			log.Info(fmt.Sprintf("Failed to subscribe to app %s\n", app.ID))
		} else {
			log.Info(fmt.Sprintf("Subscribed to app %s\n", app.ID))
		}
	}

	for {
		select {
		// case err := <-sub.Err(): // TODO: capture subscription errors
		// 	es.errs <- err
		case vLog := <-appEvents:
			log.Info(fmt.Sprintf("Witnessed tx %s on app %s\n", vLog.TxHash.Hex(), vLog.Address.Hex()))
			eventData := types.NewEventData(vLog.Address, vLog)
			es.logs <- eventData
		}
	}
}

func (es Listener) buildSubscriptionFilter(app types.Application) ethereum.FilterQuery {
	contractAddress := common.HexToAddress(app.ID)
	appEventSignature := app.ABI.Events[types.EventName].ID.Hex()
	appEventTopic := common.HexToHash(appEventSignature)

	return ethereum.FilterQuery{
		Addresses: []common.Address{contractAddress},
		Topics:    [][]common.Hash{{appEventTopic}},
	}
}

// Route packages tx data as a packet and relays it to the bridge
func (er Router) Route(eventData types.EventData) error {

	appAddress := eventData.Contract.Bytes()
	var appID [32]byte
	copy(appID[:], appAddress)

	packet, err := er.buildPacket(eventData.Contract, eventData.Data)
	if err != nil {
		return err
	}

	err = er.sendPacket(appID, packet)
	if err != nil {
		return err
	}

	return nil
}
