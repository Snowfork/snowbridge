package ethereum

import (
	"context"
	"fmt"
	"math/big"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/prover"
	"github.com/spf13/viper"

	ctypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/ethereum/go-ethereum/accounts/abi"
)

type Writer struct {
	conn *Connection
	abi  abi.ABI
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

func NewWriter(conn *Connection, stop <-chan int) (*Writer, error) {
	contractABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(RawABI))))
	if err != nil {
		return nil, err
	}

	return &Writer{
		conn: conn,
		abi:  contractABI,
		stop: stop,
	}, nil
}

func (wr *Writer) Start() error {
	log.Debug("Starting writer")
	return nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) Write(appName string, data []byte) error {

	// Get address of ethereum app
	appHexAddr := viper.GetString(strings.Join([]string{"ethereum", "apps", appName}, "."))
	appAddress := common.HexToAddress(appHexAddr)

	log.WithFields(log.Fields{
		"type":            appName,
		"contractAddress": appHexAddr,
	})
	log.Info("Submitting message to Ethereum")

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(data, wr.conn.kp.PrivateKey())
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

	txData, err := wr.abi.Pack("submit", data, proof.Signature)
	if err != nil {
		return err
	}

	tx := ctypes.NewTransaction(nonce, appAddress, value, gasLimit, gasPrice, txData)
	signedTx, err := ctypes.SignTx(tx, ctypes.HomesteadSigner{}, wr.conn.kp.PrivateKey())
	if err != nil {
		return err
	}

	err = wr.conn.client.SendTransaction(context.Background(), signedTx)
	if err != nil {
		log.WithFields(log.Fields{
			"txHash":          signedTx.Hash().Hex(),
			"contractAddress": appAddress.Hex(),
			"nonce":           nonce,
			"gasLimit":        gasLimit,
			"gasPrice":        gasPrice,
		}).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(log.Fields{
		"txHash":          signedTx.Hash().Hex(),
		"contractAddress": appAddress.Hex(),
	}).Info("Transaction submitted")

	return nil
}
