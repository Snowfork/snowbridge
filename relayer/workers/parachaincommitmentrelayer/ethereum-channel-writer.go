package parachaincommitmentrelayer

import (
	"context"
	"fmt"
	"math/big"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
)

type EthereumChannelWriter struct {
	config                     *ethereum.Config
	conn                       *ethereum.Connection
	basicInboundChannel        *inbound.BasicInboundChannel
	incentivizedInboundChannel *inbound.IncentivizedInboundChannel
	messages                   <-chan interface{}
	log                        *logrus.Entry
}

func NewEthereumChannelWriter(
	config *ethereum.Config,
	conn *ethereum.Connection,
	messages <-chan interface{},
	log *logrus.Entry,
) (*EthereumChannelWriter, error) {
	return &EthereumChannelWriter{
		config:                     config,
		conn:                       conn,
		basicInboundChannel:        nil,
		incentivizedInboundChannel: nil,
		messages:                   messages,
		log:                        log,
	}, nil
}

func (wr *EthereumChannelWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	basic, err := inbound.NewBasicInboundChannel(common.HexToAddress(wr.config.Channels.Basic.Inbound), wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.basicInboundChannel = basic

	incentivized, err := inbound.NewIncentivizedInboundChannel(common.HexToAddress(wr.config.Channels.Incentivized.Inbound), wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.incentivizedInboundChannel = incentivized

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
	options := bind.TransactOpts{
		From:     wr.conn.GetKP().CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 2000000,
	}

	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case message := <-wr.messages:
			switch value := message.(type) {
			case chain.SubstrateOutboundBasicMessage:
				err := wr.WriteBasicChannel(&options, &value)
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
					return err
				}
			case chain.SubstrateOutboundIncentivizedMessage:
				err := wr.WriteIncentivizedChannel(&options, &value)
				if err != nil {
					wr.log.WithError(err).Error("Error submitting message to ethereum")
					return err
				}
			default:
				return fmt.Errorf("Unknown message type")
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
func (wr *EthereumChannelWriter) WriteBasicChannel(
	options *bind.TransactOpts,
	msg *chain.SubstrateOutboundBasicMessage,
) error {
	var messages []inbound.BasicInboundChannelMessage
	for _, m := range msg.Messages {
		messages = append(messages,
			inbound.BasicInboundChannelMessage{
				Target:  m.Target,
				Nonce:   m.Nonce,
				Payload: m.Payload,
			},
		)
	}

	tx, err := wr.basicInboundChannel.Submit(options, messages, msg.Commitment,
		[32]byte{}, big.NewInt(0), big.NewInt(0), [][32]byte{})
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash":  tx.Hash().Hex(),
		"channel": "Basic",
	}).Info("Transaction submitted")

	return nil
}

func (wr *EthereumChannelWriter) WriteIncentivizedChannel(
	options *bind.TransactOpts,
	msg *chain.SubstrateOutboundIncentivizedMessage,
) error {
	var messages []inbound.IncentivizedInboundChannelMessage
	for _, m := range msg.Messages {
		messages = append(messages,
			inbound.IncentivizedInboundChannelMessage{
				Target:  m.Target,
				Nonce:   m.Nonce,
				Fee:     m.Fee.Int,
				Payload: m.Payload,
			},
		)
	}

	tx, err := wr.incentivizedInboundChannel.Submit(options, messages, msg.Commitment,
		[32]byte{}, big.NewInt(0), big.NewInt(0), [][32]byte{})
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash":  tx.Hash().Hex(),
		"channel": "Incentivized",
	}).Info("Transaction submitted")

	return nil
}
