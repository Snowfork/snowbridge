// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"strings"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Writer struct {
	conn      *Connection
	contracts map[substrate.ChannelID]*inbound.Contract
	messages  <-chan []chain.Message
	log       *logrus.Entry
}

func NewWriter(conn *Connection, messages <-chan []chain.Message, contracts map[substrate.ChannelID]*inbound.Contract, log *logrus.Entry) (*Writer, error) {
	return &Writer{
		conn:      conn,
		contracts: contracts,
		messages:  messages,
		log:       log,
	}, nil
}

func (wr *Writer) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return wr.writeLoop(ctx)
	})
	return nil
}

func (wr *Writer) onDone(ctx context.Context) error {
	wr.log.Info("Shutting down writer...")
	// Avoid deadlock if a listener is still trying to send to a channel
	for range wr.messages {
		wr.log.Debug("Discarded message")
	}
	return ctx.Err()
}

func (wr *Writer) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case msgs := <-wr.messages:
			for _, msg := range msgs {
				concreteMsg, ok := msg.(chain.SubstrateOutboundMessage)
				if !ok {
					return fmt.Errorf("Invalid message")
				}

				err := wr.Write(ctx, &concreteMsg)
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
				}
			}
		}
	}
}

func (wr *Writer) signerFn(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
	signedTx, err := types.SignTx(tx, types.HomesteadSigner{}, wr.conn.kp.PrivateKey())
	if err != nil {
		return  nil, err
	}
	return signedTx, nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) Write(ctx context.Context, msg *chain.SubstrateOutboundMessage) error {
	contract := wr.contracts[msg.ChannelID]
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts {
		From: wr.conn.kp.CommonAddress(),
		Signer: wr.signerFn,
		Context: ctx,
	}

	var messages []inbound.InboundChannelMessage
	for _, m := range msg.Commitment {
		messages = append(messages,
			inbound.InboundChannelMessage{
				Target: m.Target,
				Nonce: m.Nonce,
				Payload: m.Payload,
			},
		)
	}

	tx, err := contract.Submit(&options, messages)
	if err != nil {
		 wr.log.WithError(err).Error("Failed to submit transaction")
		 return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash":        tx.Hash().Hex(),
	}).Info("Transaction submitted")

	return nil
}

func loadContractABI(abiString string) (*abi.ABI, error) {
	contractABI, err := abi.JSON(strings.NewReader(abiString))
	if err != nil {
		return nil, err
	}

	return &contractABI, nil
}