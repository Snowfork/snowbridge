// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	log "github.com/sirupsen/logrus"
)

type Connection struct {
	endpoint string
	kp       *secp256k1.Keypair
	client   *ethclient.Client
	chainID  *big.Int
}

func NewConnection(endpoint string, kp *secp256k1.Keypair) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
	}
}

func (co *Connection) Connect(ctx context.Context) error {
	client, err := ethclient.Dial(co.endpoint)
	if err != nil {
		return err
	}

	chainID, err := client.NetworkID(ctx)
	if err != nil {
		return err
	}

	log.WithFields(logrus.Fields{
		"endpoint": co.endpoint,
		"chainID":  chainID,
	}).Info("Connected to chain")

	co.client = client
	co.chainID = chainID

	return nil
}

func (co *Connection) Close() {
	if co.client != nil {
		co.client.Close()
	}
}

func (co *Connection) GetClient() *ethclient.Client {
	return co.client
}

func (co *Connection) GetKP() *secp256k1.Keypair {
	return co.kp
}

func (co *Connection) ChainID() *big.Int {
	return co.chainID
}
