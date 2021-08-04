package parachain

import (
	"context"
	"math/big"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v3/types"

	log "github.com/sirupsen/logrus"
)

type EthereumChannelWriter struct {
	config                     *SinkConfig
	conn                       *ethereum.Connection
	basicInboundChannel        *basic.BasicInboundChannel
	incentivizedInboundChannel *incentivized.IncentivizedInboundChannel
	messagePackages            <-chan MessagePackage
}

func NewEthereumChannelWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	messagePackages <-chan MessagePackage,
) (*EthereumChannelWriter, error) {
	return &EthereumChannelWriter{
		config:                     config,
		conn:                       conn,
		basicInboundChannel:        nil,
		incentivizedInboundChannel: nil,
		messagePackages:            messagePackages,
	}, nil
}

func (wr *EthereumChannelWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	var address common.Address

	address = common.HexToAddress(wr.config.Contracts.BasicInboundChannel)
	basic, err := basic.NewBasicInboundChannel(address, wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.basicInboundChannel = basic

	address = common.HexToAddress(wr.config.Contracts.IncentivizedInboundChannel)
	incentivized, err := incentivized.NewIncentivizedInboundChannel(address, wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.incentivizedInboundChannel = incentivized

	eg.Go(func() error {
		return wr.writeMessagesLoop(ctx)
	})

	return nil
}

func (wr *EthereumChannelWriter) makeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := wr.conn.ChainID()
	keypair := wr.conn.GetKP()

	options := bind.TransactOpts{
		From: keypair.CommonAddress(),
		Signer: func(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
			return types.SignTx(tx, types.NewLondonSigner(chainID), keypair.PrivateKey())
		},
		Context: ctx,
	}

	if wr.config.Ethereum.GasFeeCap > 0 {
		fee := big.NewInt(0)
		fee.SetUint64(wr.config.Ethereum.GasFeeCap)
		options.GasFeeCap = fee
	}

	if wr.config.Ethereum.GasTipCap > 0 {
		tip := big.NewInt(0)
		tip.SetUint64(wr.config.Ethereum.GasTipCap)
		options.GasTipCap = tip
	}

	if wr.config.Ethereum.GasLimit > 0 {
		options.GasLimit = wr.config.Ethereum.GasLimit
	}

	return &options
}

func (wr *EthereumChannelWriter) writeMessagesLoop(ctx context.Context) error {
	options := wr.makeTxOpts(ctx)
	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down ethereum writer")
			// Drain messages to avoid deadlock
			for len(wr.messagePackages) > 0 {
				<-wr.messagePackages
			}
			return nil
		case messagePackage := <-wr.messagePackages:
			err := wr.WriteChannel(options, &messagePackage)
			if err != nil {
				log.WithError(err).Error("Error submitting message to ethereum")
				return err
			}
		}
	}
}

func (wr *EthereumChannelWriter) signerFn(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
	chainID := wr.conn.ChainID()
	signedTx, err := types.SignTx(tx, types.NewLondonSigner(chainID), wr.conn.GetKP().PrivateKey())
	if err != nil {
		return nil, err
	}
	return signedTx, nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumChannelWriter) WriteBasicChannel(
	options *bind.TransactOpts,
	msgPackage *MessagePackage,
	msgs []parachain.BasicOutboundChannelMessage,
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

	paraheadPartial := basic.ParachainLightClientOwnParachainHeadPartial{
		ParentHash:     msgPackage.paraHead.ParentHash,
		Number:         uint32(msgPackage.paraHead.Number),
		StateRoot:      msgPackage.paraHead.StateRoot,
		ExtrinsicsRoot: msgPackage.paraHead.ExtrinsicsRoot,
	}
	beefyMMRLeafPartial := basic.ParachainLightClientBeefyMMRLeafPartial{
		ParentNumber:         uint32(msgPackage.mmrProof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           msgPackage.mmrProof.Leaf.ParentNumberAndHash.Hash,
		NextAuthoritySetId:   uint64(msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.Root,
	}
	// TODO: assess this - We assume no pruning, so one leaf for each block
	beefyLeafCount := int64(msgPackage.mmrProof.Leaf.ParentNumberAndHash.ParentNumber)
	// TODO: assess this - We assume we are relaying the newest leaf
	beefyMMRLeafIndex := beefyLeafCount - 1
	var beefyMMRProof [][32]byte
	for _, item := range msgPackage.mmrProof.Proof.Items {
		beefyMMRProof = append(beefyMMRProof, [32]byte(item))
	}
	paraHeadProof := basic.ParachainLightClientParachainHeadProof{
		Pos:   big.NewInt(int64(msgPackage.paraHeadProofPos)),
		Width: big.NewInt(int64(msgPackage.paraHeadProofWidth)),
		Proof: msgPackage.paraHeadProof,
	}

	err := wr.logBasicTx(messages, paraheadPartial, paraHeadProof, msgPackage.paraHeadProofRoot,
		beefyMMRLeafPartial, beefyMMRLeafIndex, beefyLeafCount, beefyMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to log transaction input")
		return err
	}

	tx, err := wr.basicInboundChannel.Submit(options, messages, paraheadPartial,
		paraHeadProof, beefyMMRLeafPartial,
		big.NewInt(beefyMMRLeafIndex), big.NewInt(beefyLeafCount), beefyMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(logrus.Fields{
		"txHash":  tx.Hash().Hex(),
		"channel": "Basic",
	}).Info("Transaction submitted")

	return nil
}

func (wr *EthereumChannelWriter) WriteIncentivizedChannel(
	options *bind.TransactOpts,
	msgPackage *MessagePackage,
	msgs []parachain.IncentivizedOutboundChannelMessage,
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

	paraheadPartial := incentivized.ParachainLightClientOwnParachainHeadPartial{
		ParentHash:     msgPackage.paraHead.ParentHash,
		Number:         uint32(msgPackage.paraHead.Number),
		StateRoot:      msgPackage.paraHead.StateRoot,
		ExtrinsicsRoot: msgPackage.paraHead.ExtrinsicsRoot,
	}
	beefyMMRLeafPartial := incentivized.ParachainLightClientBeefyMMRLeafPartial{
		ParentNumber:         uint32(msgPackage.mmrProof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           msgPackage.mmrProof.Leaf.ParentNumberAndHash.Hash,
		NextAuthoritySetId:   uint64(msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: msgPackage.mmrProof.Leaf.BeefyNextAuthoritySet.Root,
	}
	// TODO: assess this - We assume no pruning, so one leaf for each block
	beefyLeafCount := int64(msgPackage.mmrProof.Leaf.ParentNumberAndHash.ParentNumber)
	// TODO: assess this - We assume we are relaying the newest leaf
	beefyMMRLeafIndex := beefyLeafCount - 1
	var beefyMMRProof [][32]byte
	for _, item := range msgPackage.mmrProof.Proof.Items {
		beefyMMRProof = append(beefyMMRProof, [32]byte(item))
	}
	paraHeadProof := incentivized.ParachainLightClientParachainHeadProof{
		Pos:   big.NewInt(int64(msgPackage.paraHeadProofPos)),
		Width: big.NewInt(int64(msgPackage.paraHeadProofWidth)),
		Proof: msgPackage.paraHeadProof,
	}

	err := wr.logIncentivizedTx(messages, paraheadPartial, paraHeadProof, msgPackage.paraHeadProofRoot,
		beefyMMRLeafPartial, beefyMMRLeafIndex, beefyLeafCount, beefyMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to log transaction input")
		return err
	}

	tx, err := wr.incentivizedInboundChannel.Submit(options, messages,
		paraheadPartial,
		paraHeadProof, beefyMMRLeafPartial,
		big.NewInt(beefyMMRLeafIndex), big.NewInt(beefyLeafCount), beefyMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(logrus.Fields{
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
		var outboundMessages []parachain.BasicOutboundChannelMessage
		err := gsrpcTypes.DecodeFromBytes(msg.commitmentData, &outboundMessages)
		if err != nil {
			log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}
		err = wr.WriteBasicChannel(options, msg, outboundMessages)
		if err != nil {
			log.WithError(err).Error("Failed to write basic channel")
			return err
		}

	}
	if msg.channelID.IsIncentivized {
		var outboundMessages []parachain.IncentivizedOutboundChannelMessage
		err := gsrpcTypes.DecodeFromBytes(msg.commitmentData, &outboundMessages)
		if err != nil {
			log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}
		err = wr.WriteIncentivizedChannel(options, msg, outboundMessages)
		if err != nil {
			log.WithError(err).Error("Failed to write incentivized channel")
			return err
		}
	}

	return nil
}
