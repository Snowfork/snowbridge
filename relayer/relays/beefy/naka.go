package beefy

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	ethTypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"golang.org/x/sync/errgroup"
)

type Naka struct {
	sinkConfig       SinkConfig
	sourceConfig     SourceConfig
	ethConn          *ethereum.Connection
	subConn          *relaychain.Connection
	beefyLightClient *beefylightclient.Contract
}

func NewNaka(
	sinkConfig SinkConfig,
	sourceConfig SourceConfig,
	ethConn *ethereum.Connection,
	subConn *relaychain.Connection,
) *Naka {
	return &Naka{
		sinkConfig: sinkConfig, sourceConfig: sourceConfig, ethConn: ethConn, subConn: subConn,
	}
}

func (n *Naka) Start(ctx context.Context, eg *errgroup.Group) error {
	address := common.HexToAddress(n.sinkConfig.Contracts.BeefyLightClient)
	contract, err := beefylightclient.NewContract(address, n.ethConn.GetClient())
	if err != nil {
		return err
	}
	n.beefyLightClient = contract

	beefyBlock, err := n.beefyLightClient.LatestBeefyBlock(nil)
	if err != nil {
		return fmt.Errorf("fetch BeefyLightClient.latestBeefyBlock: %w", err)
	}

	beefyBlockHash, err := n.subConn.API().RPC.Chain.GetBlockHash(beefyBlock)
	if err != nil {
		return fmt.Errorf("fetch hash for block %v: %w", beefyBlockHash.Hex(), err)
	}

	nextValidatorSet, err := n.beefyLightClient.NextValidatorSet(nil)
	if err != nil {
		return fmt.Errorf("fetch BeefyLightClient.nextValidatorSet: %w", err)
	}

	// we can use any block except the latest beefy block

	if beefyBlock > 0 {
		blockToProve := beefyBlock - 1
		proof, err := n.subConn.GenerateProofForBlock(blockToProve, beefyBlockHash, n.sourceConfig.BeefyActivationBlock)
		if err != nil {
			return fmt.Errorf("proof generation for %v: %w", blockToProve, err)
		}

		if uint64(proof.Leaf.BeefyNextAuthoritySet.ID) == nextValidatorSet.Id.Uint64()+1 {
			if err := n.updateNextValidatorSet(ctx, proof); err != nil {
				return err
			}
		}
	}

	eg.Go(func() error {
		err := n.watchNewSessionEvents(ctx)
		log.Debug("Shutting down NewSession event watcher")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return fmt.Errorf("service Naka: %w", err)
		}
		return nil
	})

	return nil
}

func (n *Naka) watchNewSessionEvents(ctx context.Context) error {
	opts := bind.WatchOpts{
		Context: ctx,
	}

	events := make(chan *beefylightclient.ContractNewSession)
	sub, err := n.beefyLightClient.WatchNewSession(&opts, events)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			return err
		case ev, ok := <-events:
			if !ok {
				return nil
			}

			beefyBlockHash, err := n.subConn.API().RPC.Chain.GetBlockHash(ev.BlockNumber)
			if err != nil {
				return fmt.Errorf("fetch hash for block %v: %w", beefyBlockHash.Hex(), err)
			}

			// we can use any block except the latest beefy block
			blockToProve := ev.BlockNumber - 1
			proof, err := n.subConn.GenerateProofForBlock(blockToProve, beefyBlockHash, n.sourceConfig.BeefyActivationBlock)
			if err != nil {
				return fmt.Errorf("proof generation for %v: %w", blockToProve, err)
			}

			if err := n.updateNextValidatorSet(ctx, proof); err != nil {
				return err
			}
		}
	}
}

func (n *Naka) updateNextValidatorSet(ctx context.Context, proof types.GenerateMMRProofResponse) error {

	p, err := merkle.ConvertToSimplifiedMMRProof(
		proof.BlockHash,
		uint64(proof.Proof.LeafIndex),
		proof.Leaf,
		uint64(proof.Proof.LeafCount),
		proof.Proof.Items,
	)
	if err != nil {
		return fmt.Errorf("simplified proof conversion for block %v: %w", proof.BlockHash.Hex(), err)
	}

	inputLeaf := beefylightclient.BeefyLightClientMMRLeaf{
		Version:              uint8(p.Leaf.Version),
		ParentNumber:         uint32(p.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           p.Leaf.ParentNumberAndHash.Hash,
		ParachainHeadsRoot:   p.Leaf.ParachainHeads,
		NextAuthoritySetId:   uint64(p.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(p.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: p.Leaf.BeefyNextAuthoritySet.Root,
	}

	merkleProofItems := [][32]byte{}
	for _, mmrProofItem := range p.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, mmrProofItem)
	}

	inputProof := beefylightclient.SimplifiedMMRProof{
		MerkleProofItems:         merkleProofItems,
		MerkleProofOrderBitField: p.MerkleProofOrder,
	}

	opts := n.makeTxOpts(ctx)

	tx, err := n.beefyLightClient.UpdateValidatorSet(opts, inputLeaf, inputProof)
	if err != nil {
		return fmt.Errorf("send transaction UpdateValidatorSet: %w", err)
	}

	log.WithField("txHash", tx.Hash().Hex()).Info("Sent UpdateValidatorSet transaction")

	return nil
}

func (n *Naka) makeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := n.ethConn.ChainID()
	keypair := n.ethConn.GetKP()

	options := bind.TransactOpts{
		From: keypair.CommonAddress(),
		Signer: func(_ common.Address, tx *ethTypes.Transaction) (*ethTypes.Transaction, error) {
			return ethTypes.SignTx(tx, ethTypes.NewLondonSigner(chainID), keypair.PrivateKey())
		},
		Context: ctx,
	}

	if n.sinkConfig.Ethereum.GasFeeCap > 0 {
		fee := big.NewInt(0)
		fee.SetUint64(n.sinkConfig.Ethereum.GasFeeCap)
		options.GasFeeCap = fee
	}

	if n.sinkConfig.Ethereum.GasTipCap > 0 {
		tip := big.NewInt(0)
		tip.SetUint64(n.sinkConfig.Ethereum.GasTipCap)
		options.GasTipCap = tip
	}

	if n.sinkConfig.Ethereum.GasLimit > 0 {
		options.GasLimit = n.sinkConfig.Ethereum.GasLimit
	}

	return &options
}
