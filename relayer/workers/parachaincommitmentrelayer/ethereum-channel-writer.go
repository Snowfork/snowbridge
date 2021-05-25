package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
)

type EthereumChannelWriter struct {
	config    *ethereum.Config
	conn      *ethereum.Connection
	contracts map[substrate.ChannelID]*inbound.Contract
	messages  <-chan []chain.Message
	log       *logrus.Entry
	beefyDB   *store.Database
}

func NewEthereumChannelWriter(config *ethereum.Config, conn *ethereum.Connection, messages <-chan []chain.Message,
	contracts map[substrate.ChannelID]*inbound.Contract,
	log *logrus.Entry) (*EthereumChannelWriter, error) {
	return &EthereumChannelWriter{
		config:    config,
		conn:      conn,
		contracts: contracts,
		messages:  messages,
		log:       log,
	}, nil
}

func (wr *EthereumChannelWriter) Start(ctx context.Context, eg *errgroup.Group) error {

	id := substrate.ChannelID{IsBasic: true}
	contract, err := inbound.NewContract(common.HexToAddress(wr.config.Channels.Basic.Inbound), wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.contracts[id] = contract

	id = substrate.ChannelID{IsIncentivized: true}
	contract, err = inbound.NewContract(common.HexToAddress(wr.config.Channels.Incentivized.Inbound), wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.contracts[id] = contract

	eg.Go(func() error {
		return wr.writeMessagesLoop(ctx)
	})

	return nil
}

func (wr *EthereumChannelWriter) onDone(ctx context.Context) error {
	wr.log.Info("Shutting down writer...")
	// Avoid deadlock if a listener is still trying to send to a channel
	for range wr.messages {
		wr.log.Debug("Discarded message")
	}
	return ctx.Err()
}

func (wr *EthereumChannelWriter) writeMessagesLoop(ctx context.Context) error {
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
					return err
				}
			}
		}
	}
}

func (wr *EthereumChannelWriter) signerFn(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
	signedTx, err := types.SignTx(tx, types.HomesteadSigner{}, wr.conn.GetKP().PrivateKey())
	if err != nil {
		return nil, err
	}
	return signedTx, nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumChannelWriter) WriteChannel(ctx context.Context, msg *chain.SubstrateOutboundMessage) error {
	contract := wr.contracts[msg.ChannelID]
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.conn.GetKP().CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 500000,
	}

	var messages []inbound.InboundChannelMessage
	for _, m := range msg.Commitment {
		// Check if block number exists in beefy database
		beefyData := wr.beefyDB.GetItemByBlockNumber(*msg.BlockNumber)
		if beefyData.Status == store.CompleteVerificationTxConfirmed {
			messages = append(messages,
				inbound.InboundChannelMessage{
					Target:  m.Target,
					Nonce:   m.Nonce,
					Payload: m.Payload,
				},
			)
		}
	}

	tx, err := contract.Submit(
		&options,
		messages,
		msg.CommitmentHash,
	)
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash":       tx.Hash().Hex(),
		"basic":        msg.ChannelID.IsBasic,
		"incentivized": msg.ChannelID.IsIncentivized,
	}).Info("Transaction submitted")

	return nil
}
