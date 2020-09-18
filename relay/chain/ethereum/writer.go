// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"math/big"
	"strings"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

type Writer struct {
	conn     *Connection
	abi      abi.ABI
	messages <-chan chain.Message
	log      *logrus.Entry
}

const RawABI = `
[
	{
		"inputs": [
			{
				"internalType": "address",
				"name": "appId",
				"type": "address"
			},
			{
				"internalType": "bytes",
				"name": "message",
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

func NewWriter(conn *Connection, messages <-chan chain.Message, log *logrus.Entry) (*Writer, error) {
	contractABI, err := abi.JSON(strings.NewReader(RawABI))

	if err != nil {
		return nil, err
	}

	return &Writer{
		conn:     conn,
		abi:      contractABI,
		messages: messages,
		log:      log,
	}, nil
}

func (wr *Writer) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return wr.writeLoop(ctx)
	})
	return nil
}

func (wr *Writer) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case msg := <-wr.messages:
			err := wr.Write(ctx, &msg)
			if err != nil {
				wr.log.WithError(err).Error("Error submitting message to ethereum")
			}
		}
	}
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) Write(ctx context.Context, msg *chain.Message) error {

	address := common.Address(msg.AppID)

	wr.log.WithFields(logrus.Fields{
		"contractAddress": address.Hex(),
	}).Info("Submitting message to Ethereum")

	nonce, err := wr.conn.client.PendingNonceAt(ctx, wr.conn.kp.CommonAddress())
	if err != nil {
		return err
	}

	value := big.NewInt(0)      // in wei (0 eth)
	gasLimit := uint64(2000000) // in units
	gasPrice, err := wr.conn.client.SuggestGasPrice(ctx)
	if err != nil {
		return err
	}

	txData, err := wr.abi.Pack("submit", msg.Payload)
	if err != nil {
		return err
	}

	tx := types.NewTransaction(nonce, address, value, gasLimit, gasPrice, txData)
	signedTx, err := types.SignTx(tx, types.HomesteadSigner{}, wr.conn.kp.PrivateKey())
	if err != nil {
		return err
	}

	err = wr.conn.client.SendTransaction(ctx, signedTx)
	if err != nil {
		wr.log.WithError(err).WithFields(logrus.Fields{
			"txHash":          signedTx.Hash().Hex(),
			"contractAddress": address.Hex(),
			"nonce":           nonce,
			"gasLimit":        gasLimit,
			"gasPrice":        gasPrice,
		}).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash":          signedTx.Hash().Hex(),
		"contractAddress": address.Hex(),
	}).Info("Transaction submitted")

	return nil
}
