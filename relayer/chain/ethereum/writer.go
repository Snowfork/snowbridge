// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"math/big"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/polkadotrelaychainbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/validatorregistry"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

const (
	PolkadotRelayChainBridge = "polkadot_relay_chain_bridge"
	ValidatorRegistry        = "validator_registry"
)

type Writer struct {
	config                   *Config
	conn                     *Connection
	db                       *store.Database
	contracts                map[substrate.ChannelID]*inbound.Contract
	polkadotRelayChainBridge *polkadotrelaychainbridge.Contract
	validatorRegistry        *validatorregistry.Contract
	messages                 <-chan []chain.Message
	beefyMessages            chan<- store.DatabaseCmd
	log                      *logrus.Entry
}

func NewWriter(config *Config, conn *Connection, db *store.Database, messages <-chan []chain.Message,
	beefyMessages chan<- store.DatabaseCmd, contracts map[substrate.ChannelID]*inbound.Contract,
	log *logrus.Entry) (*Writer, error) {
	return &Writer{
		config:        config,
		conn:          conn,
		db:            db,
		contracts:     contracts,
		messages:      messages,
		beefyMessages: beefyMessages,
		log:           log,
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

	polkadotRelayChainBridgeContract, err := polkadotrelaychainbridge.NewContract(common.HexToAddress(wr.config.Contracts.PolkadotRelayChainBridge), wr.conn.client)
	if err != nil {
		return err
	}
	wr.polkadotRelayChainBridge = polkadotRelayChainBridgeContract

	validatorRegistryContract, err := validatorregistry.NewContract(common.HexToAddress(wr.config.Contracts.ValidatorRegistry), wr.conn.client)
	if err != nil {
		return err
	}
	wr.validatorRegistry = validatorRegistryContract

	eg.Go(func() error {
		return wr.writeMessagesLoop(ctx)
	})

	// TODO: resolve tx nonce conflict edge case
	eg.Go(func() error {
		return wr.writeBeefyCreateLoop(ctx)
	})

	eg.Go(func() error {
		return wr.writeBeefyCompleteLoop(ctx)
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
		}
	}
}

func (wr *Writer) writeBeefyCreateLoop(ctx context.Context) error {
	ticker := time.NewTicker(5 * time.Second)
	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case <-ticker.C:
			// Send initial verification txs for witnessed commitments
			witnessedItems := wr.db.GetItemsByStatus(store.CommitmentWitnessed)
			for _, item := range witnessedItems {
				err := wr.WriteNewSignatureCommitment(ctx, item, 0) // TODO: match validator address to index
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
				}
			}
		}
	}
}

func (wr *Writer) writeBeefyCompleteLoop(ctx context.Context) error {
	ticker := time.NewTicker(11 * time.Second)
	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case <-ticker.C:
			// Send complete verification txs for items that are ready to complete
			completeItems := wr.db.GetItemsByStatus(store.ReadyToComplete)
			for _, item := range completeItems {
				err := wr.WriteCompleteSignatureCommitment(ctx, item)
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

func (wr *Writer) WriteNewSignatureCommitment(ctx context.Context, item *store.BeefyItem, valIndex int) error {
	beefy, err := item.ToBeefy()
	if err != nil {
		wr.log.WithError(err).Error("Error converting database item to Beefy")
	}

	msg, err := beefy.BuildNewSignatureCommitmentMessage(valIndex)
	if err != nil {
		return err
	}

	// inSet, err := wr.CheckValidatorInSet(ctx, msg.ValidatorPublicKey, msg.ValidatorPosition, msg.ValidatorPublicKeyMerkleProof)
	// if err != nil {
	// 	return err
	// }
	// if !inSet {
	// 	return fmt.Errorf("validator address merkle proof failed verification")
	// }

	contract := wr.polkadotRelayChainBridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	blockNumber, err := wr.conn.client.BlockNumber(ctx)
	if err != nil {
		return err
	}

	options := bind.TransactOpts{
		From:     wr.conn.GetKeyPair().CommonAddress(),
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
		"txHash":      tx.Hash().Hex(),
		"blockNumber": blockNumber,
	}).Info("New Signature Commitment transaction submitted")

	// Update item's status in database
	wr.log.Info("2: Updating item status from 'WitnessedCommitment' to 'InitialVerificationTxSent'")
	instructions := map[string]interface{}{
		"status":                       store.InitialVerificationTxSent,
		"initial_verification_tx_hash": tx.Hash(),
	}
	updateCmd := store.NewDatabaseCmd(item, true, instructions)
	wr.beefyMessages <- updateCmd

	return nil
}

// WriteCompleteSignatureCommitment sends a CompleteSignatureCommitment tx to the PolkadotRelayChainBridge contract
func (wr *Writer) WriteCompleteSignatureCommitment(ctx context.Context, item *store.BeefyItem) error {
	beefy, err := item.ToBeefy()
	if err != nil {
		wr.log.WithError(err).Error("Error converting database item to Beefy")
	}

	msg, err := beefy.BuildCompleteSignatureCommitmentMessage()
	if err != nil {
		return err
	}

	contract := wr.polkadotRelayChainBridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	blockNumber, err := wr.conn.client.BlockNumber(ctx)
	if err != nil {
		return err
	}

	currAccNonce, err := wr.conn.client.NonceAt(ctx, wr.conn.GetKeyPair().CommonAddress(), big.NewInt(int64(blockNumber)))
	if err != nil {
		return err
	}

	options := bind.TransactOpts{
		From:     wr.conn.GetKeyPair().CommonAddress(),
		Nonce:    big.NewInt(int64(currAccNonce)),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 500000,
	}

	tx, err := contract.CompleteSignatureCommitment(&options, msg.ID, msg.Payload, msg.RandomSignatureCommitments,
		msg.RandomSignatureBitfieldPositions, msg.RandomValidatorAddresses, msg.RandomPublicKeyMerkleProofs)

	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("5. Complete Signature Commitment transaction submitted")

	// TODO: delete from database

	return nil
}
