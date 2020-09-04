package ethereum

import (
	log "github.com/sirupsen/logrus"

	"sync"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"

	"github.com/ethereum/go-ethereum/common"
	"github.com/spf13/viper"

	"context"
	"fmt"
	"math/big"

	"strings"

	ctypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"

	ethKeys "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	"github.com/snowfork/polkadot-ethereum/prover"

	"github.com/ethereum/go-ethereum/accounts/abi"
)

const RawABI = `
[
	{
		"inputs": [
			  {
				"internalType": "bytes",
				"name": "data",
				"type": "bytes"
			  },
			  {
				"internalType": "bytes",
				"name": "signature",
				"type": "bytes"
			  }
		],
		"name": "submit",
		"outputs": [],
		"stateMutability": "nonpayable",
		"type": "function"
	  }
]
`

// EthChain streams the Ethereum blockchain and routes tx data packets
type Chain struct {
	Streamer    Streamer
	keybase     *ethKeys.Keypair
	client      *ethclient.Client
	verifier    common.Address
	contractABI abi.ABI
}

// NewEthChain initializes a new instance of EthChain
func NewChain(websocketURL string, keybase *ethKeys.Keypair, verifier common.Address) (*Chain, error) {

	// Load ethereum ABIs
	streamer := NewStreamer(viper.GetString("ethereum.endpoint"), registryPath())

	ethKeybase, err := ethKeys.NewKeypairFromString(viper.GetString("ethereum.private_key"))
	if err != nil {
		return nil, err
	}

	client, err := ethclient.Dial(websocketURL)
	if err != nil {
		return nil, err
	}

	contractABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(RawABI))))
	if err != nil {
		return nil, err
	}

	return Chain{
		Streamer:    streamer,
		keybase:     keybase,
		client:      client,
		verifier:    verifier,
		contractABI: contractABI,
	}, nil
}

func (ec Chain) Start(wg *sync.WaitGroup) error {
	defer wg.Done()

	errors := make(chan error, 0)
	events := make(chan types.EventData, 0)

	go ec.Streamer.Start(events, errors)

	for {
		select {
		case err := <-errors:
			log.Error(err)
		case event := <-events:
			err := ec.Router.Route(event)
			if err != nil {
				log.Error(err)
			}
		}
	}
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (ec Chain) Submit(appName string, data []byte) error {

	log.Info("Submitting ", appName, " message to Ethereum")

	// Get address of ethereum app
	appHexAddr := viper.GetString(strings.Join([]string{"ethereum", "apps", appName}, "."))
	appAddress := common.HexToAddress(appHexAddr)
	log.Info("App Address: ", appHexAddr)

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(data, ec.keybase.PrivateKey())
	if err != nil {
		return err
	}

	nonce, err := ec.client.PendingNonceAt(context.Background(), ec.keybase.CommonAddress())
	if err != nil {
		return err
	}

	value := big.NewInt(0)      // in wei (0 eth)
	gasLimit := uint64(2000000) // in units
	gasPrice, err := ec.client.SuggestGasPrice(context.Background())
	if err != nil {
		return err
	}

	txData, err := ec.contractABI.Pack("submit", data, proof.Signature)
	if err != nil {
		return err
	}

	tx := ctypes.NewTransaction(nonce, appAddress, value, gasLimit, gasPrice, txData)
	signedTx, err := ctypes.SignTx(tx, ctypes.HomesteadSigner{}, ec.keybase.PrivateKey())
	if err != nil {
		return err
	}

	err = ec.client.SendTransaction(context.Background(), signedTx)
	if err != nil {
		return err
	}

	log.Info("tx sent: ", signedTx.Hash().Hex())
	return nil
}
