// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Writer struct {
	config            *Config
	conn              *Connection
	db                *store.Database
	contracts         map[substrate.ChannelID]*inbound.Contract
	lightClientBridge *lightclientbridge.Contract
	messages          <-chan []chain.Message
	databaseMessages  chan<- store.DatabaseCmd
	beefyMessages     <-chan store.BeefyRelayInfo
	log               *logrus.Entry
}

func NewWriter(config *Config, conn *Connection, db *store.Database, messages <-chan []chain.Message,
	databaseMessages chan<- store.DatabaseCmd, beefyMessages <-chan store.BeefyRelayInfo,
	contracts map[substrate.ChannelID]*inbound.Contract,
	log *logrus.Entry) (*Writer, error) {
	return &Writer{
		config:           config,
		conn:             conn,
		db:               db,
		contracts:        contracts,
		messages:         messages,
		databaseMessages: databaseMessages,
		beefyMessages:    beefyMessages,
		log:              log,
	}, nil
}

func (wr *Writer) Start(ctx context.Context, eg *errgroup.Group) error {

	id := substrate.ChannelID{IsBasic: true}
	contract, err := inbound.NewContract(common.HexToAddress(wr.config.Channels.Basic.Inbound), wr.conn.client)
	if err != nil {
		return err
	}
	wr.contracts[id] = contract

	id = substrate.ChannelID{IsIncentivized: true}
	contract, err = inbound.NewContract(common.HexToAddress(wr.config.Channels.Incentivized.Inbound), wr.conn.client)
	if err != nil {
		return err
	}
	wr.contracts[id] = contract

	lightClientBridgeContract, err := lightclientbridge.NewContract(common.HexToAddress(wr.config.LightClientBridge), wr.conn.client)
	if err != nil {
		return err
	}
	wr.lightClientBridge = lightClientBridgeContract

	eg.Go(func() error {
		return wr.writeMessagesLoop(ctx)
	})

	return nil
}

func (wr *Writer) onDone(ctx context.Context) error {
	wr.log.Info("Shutting down writer...")
	// Avoid deadlock if a listener is still trying to send to a channel
	for range wr.messages {
		wr.log.Debug("Discarded message")
	}
	for range wr.beefyMessages {
		wr.log.Debug("Discarded BEEFY message")
	}
	return ctx.Err()
}

func (wr *Writer) writeMessagesLoop(ctx context.Context) error {
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

				err := wr.WriteChannel(ctx, &concreteMsg)
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
				}
			}
		case msg := <-wr.beefyMessages:
			switch msg.Status {
			case store.CommitmentWitnessed:
				err := wr.WriteNewSignatureCommitment(ctx, msg, 0) // TODO: pick val addr
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
				}
			case store.ReadyToComplete:
				err := wr.WriteCompleteSignatureCommitment(ctx, msg)
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
		return nil, err
	}
	return signedTx, nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) WriteChannel(ctx context.Context, msg *chain.SubstrateOutboundMessage) error {
	contract := wr.contracts[msg.ChannelID]
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.conn.kp.CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 500000,
	}

	var messages []inbound.InboundChannelMessage
	for _, m := range msg.Commitment {
		messages = append(messages,
			inbound.InboundChannelMessage{
				Target:  m.Target,
				Nonce:   m.Nonce,
				Payload: m.Payload,
			},
		)
	}

	tx, err := contract.Submit(&options, messages, msg.CommitmentHash)
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("Transaction submitted")

	return nil
}

func (wr *Writer) WriteNewSignatureCommitment(ctx context.Context, info store.BeefyRelayInfo, valIndex int) error {
	beefyJustification, err := info.ToBeefyJustification()
	if err != nil {
		return fmt.Errorf("Error converting BeefyRelayInfo to BeefyJustification: %s", err.Error())
	}

	msg, err := beefyJustification.BuildNewSignatureCommitmentMessage(valIndex)
	if err != nil {
		return err
	}

	contract := wr.lightClientBridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.conn.kp.CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 5000000,
	}

	tx, err := contract.NewSignatureCommitment(&options, msg.Payload,
		msg.ValidatorClaimsBitfield, msg.ValidatorSignatureCommitment,
		msg.ValidatorPosition, msg.ValidatorPublicKey, msg.ValidatorPublicKeyMerkleProof)
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("New Signature Commitment transaction submitted")

	wr.log.Info("1: Creating item in Database with status 'InitialVerificationTxSent'")
	info.Status = store.InitialVerificationTxSent
	info.InitialVerificationTxHash = tx.Hash()
	cmd := store.NewDatabaseCmd(&info, store.Create, nil)
	wr.databaseMessages <- cmd

	return nil
}

// WriteCompleteSignatureCommitment sends a CompleteSignatureCommitment tx to the LightClientBridge contract
func (wr *Writer) WriteCompleteSignatureCommitment(ctx context.Context, info store.BeefyRelayInfo) error {
	beefyJustification, err := info.ToBeefyJustification()
	if err != nil {
		return fmt.Errorf("Error converting BeefyRelayInfo to BeefyJustification: %s", err.Error())
	}

	msg, err := beefyJustification.BuildCompleteSignatureCommitmentMessage()
	if err != nil {
		return err
	}

	contract := wr.lightClientBridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.conn.kp.CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 500000,
	}

	tx, err := contract.CompleteSignatureCommitment(&options, msg.ID, msg.Payload, msg.Signatures,
		msg.ValidatorPositions, msg.ValidatorPublicKeys, msg.ValidatorPublicKeyMerkleProofs)

	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("Complete Signature Commitment transaction submitted")

	// Update item's status in database
	wr.log.Info("4: Updating item status from 'ReadyToComplete' to 'CompleteVerificationTxSent'")
	instructions := map[string]interface{}{
		"status":                        store.CompleteVerificationTxSent,
		"complete_verification_tx_hash": tx.Hash(),
	}
	updateCmd := store.NewDatabaseCmd(&info, store.Update, instructions)
	wr.databaseMessages <- updateCmd

	// TODO: delete from database after confirming
	return nil
}
