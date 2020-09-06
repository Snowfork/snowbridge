package ethereum

import (
	"context"
	"fmt"
	"math/big"
	"strings"

	"github.com/spf13/viper"
	log "github.com/sirupsen/logrus"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/accounts/abi"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/prover"
)

type Writer struct {
	conn *Connection
	abi  abi.ABI
	messages <-chan chain.Message
	stop <-chan int
}

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

func NewWriter(conn *Connection, messages <-chan chain.Message, stop <-chan int) (*Writer, error) {
	contractABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(RawABI))))
	if err != nil {
		return nil, err
	}

	return &Writer{
		conn: conn,
		abi:  contractABI,
		messages: messages,
		stop: stop,
	}, nil
}

func (wr *Writer) Start() error {
	log.Debug("Starting writer")

	go func() {
		wr.writeLoop()
	}()

	return nil
}

func (wr *Writer) writeLoop() {
	for {
		select {
		case <-wr.stop:
			return
		case msg := <-wr.messages:
			err := wr.write(&msg)
			if err != nil {
				log.WithFields(log.Fields{
					"error": err,
				}).Error("Error submitting message to ethereum")
			}
		}
	}
}

// TODO: this is an interim standin until https://github.com/Snowfork/polkadot-ethereum/issues/61 lands
func (wr *Writer) lookupAppAddress(appid [32]byte) common.Address {
	var appName string
	if appid == chain.EthAppID {
		appName = "eth"
	} else if appid == chain.Erc20AppID {
		appName = "erc20"
	} else {
		panic("should not reach here")
	}

	hexaddr := viper.GetString(strings.Join([]string{"ethereum", "apps", appName}, "."))
	return common.HexToAddress(hexaddr)
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) write(msg *chain.Message) error {

	address := wr.lookupAppAddress(msg.AppID)

	log.WithFields(log.Fields{
		"contractAddress": address.Hex(),
	})
	log.Info("Submitting message to Ethereum")

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(msg.Payload, wr.conn.kp.PrivateKey())
	if err != nil {
		return err
	}

	nonce, err := wr.conn.client.PendingNonceAt(context.Background(), wr.conn.kp.CommonAddress())
	if err != nil {
		return err
	}

	value := big.NewInt(0)      // in wei (0 eth)
	gasLimit := uint64(2000000) // in units
	gasPrice, err := wr.conn.client.SuggestGasPrice(context.Background())
	if err != nil {
		return err
	}

	txData, err := wr.abi.Pack("submit", msg.Payload, proof.Signature)
	if err != nil {
		return err
	}

	tx := types.NewTransaction(nonce, address, value, gasLimit, gasPrice, txData)
	signedTx, err := types.SignTx(tx, types.HomesteadSigner{}, wr.conn.kp.PrivateKey())
	if err != nil {
		return err
	}

	err = wr.conn.client.SendTransaction(context.Background(), signedTx)
	if err != nil {
		log.WithFields(log.Fields{
			"txHash":          signedTx.Hash().Hex(),
			"contractAddress": address.Hex(),
			"nonce":           nonce,
			"gasLimit":        gasLimit,
			"gasPrice":        gasPrice,
		}).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(log.Fields{
		"txHash":          signedTx.Hash().Hex(),
		"contractAddress": address.Hex(),
	}).Info("Transaction submitted")

	return nil
}
