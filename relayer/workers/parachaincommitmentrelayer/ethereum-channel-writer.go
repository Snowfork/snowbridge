package parachaincommitmentrelayer

import (
	"context"
	"math/big"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/basic"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/incentivized"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v2/types"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type EthereumChannelWriter struct {
	config                     *ethereum.Config
	conn                       *ethereum.Connection
	basicInboundChannel        *basic.BasicInboundChannel
	incentivizedInboundChannel *incentivized.IncentivizedInboundChannel
	messagePackages            <-chan MessagePackage
	log                        *logrus.Entry
}

func NewEthereumChannelWriter(
	config *ethereum.Config,
	conn *ethereum.Connection,
	messagePackages <-chan MessagePackage,
	log *logrus.Entry,
) (*EthereumChannelWriter, error) {
	return &EthereumChannelWriter{
		config:                     config,
		conn:                       conn,
		basicInboundChannel:        nil,
		incentivizedInboundChannel: nil,
		messagePackages:            messagePackages,
		log:                        log,
	}, nil
}

func (wr *EthereumChannelWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	basic, err := basic.NewBasicInboundChannel(common.HexToAddress(wr.config.Channels.Basic.Inbound), wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.basicInboundChannel = basic

	incentivized, err := incentivized.NewIncentivizedInboundChannel(common.HexToAddress(wr.config.Channels.Incentivized.Inbound), wr.conn.GetClient())
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
	for range wr.messagePackages {
		wr.log.Debug("Discarded message package")
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
		case messagePackage := <-wr.messagePackages:
			err := wr.WriteChannel(&options, &messagePackage)
			if err != nil {
				wr.log.WithError(err).Error("Error submitting message to ethereum")
				return err
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
	commitment gsrpcTypes.H256,
	msgs []substrate.BasicOutboundChannelMessage,
) error {
	var messages []basic.BasicInboundChannelMessage
	for _, m := range msgs {
		messages = append(messages,
			basic.BasicInboundChannelMessage{
				Target:  m.Target,
				Nonce:   m.Nonce,
				Payload: m.Payload,
			},
		)
	}

	tx, err := wr.basicInboundChannel.Submit(options, messages, basic.ParachainLightClientOwnParachainHeadPartial{},
		[][32]byte{}, basic.ParachainLightClientBeefyMMRLeafPartial{},
		big.NewInt(0), big.NewInt(0), [][32]byte{})
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
	commitment gsrpcTypes.H256,
	msgs []substrate.IncentivizedOutboundChannelMessage,
) error {
	var messages []incentivized.IncentivizedInboundChannelMessage
	for _, m := range msgs {
		messages = append(messages,
			incentivized.IncentivizedInboundChannelMessage{
				Target:  m.Target,
				Nonce:   m.Nonce,
				Fee:     m.Fee.Int,
				Payload: m.Payload,
			},
		)
	}

	tx, err := wr.incentivizedInboundChannel.Submit(options, messages,
		incentivized.ParachainLightClientOwnParachainHeadPartial{},
		[][32]byte{}, incentivized.ParachainLightClientBeefyMMRLeafPartial{},
		big.NewInt(0), big.NewInt(0), [][32]byte{})
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

func (wr *EthereumChannelWriter) WriteChannel(
	options *bind.TransactOpts,
	msg *MessagePackage,
) error {
	if msg.channelID.IsBasic {
		var outboundMessages []chainTypes.BasicOutboundChannelMessage
		err := gsrpcTypes.DecodeFromBytes(msg.commitmentData, &outboundMessages)
		if err != nil {
			wr.log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}
		wr.WriteBasicChannel(options, msg.commitmentHash, outboundMessages)

	}
	if msg.channelID.IsIncentivized {
		var outboundMessages []chainTypes.IncentivizedOutboundChannelMessage
		err := gsrpcTypes.DecodeFromBytes(msg.commitmentData, &outboundMessages)
		if err != nil {
			wr.log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}
		wr.WriteIncentivizedChannel(options, msg.commitmentHash, outboundMessages)
	}

	return nil
}
