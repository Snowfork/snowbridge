package beefy

import (
	"context"
	"errors"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"golang.org/x/sync/errgroup"
)

type Naka struct {
	sinkConfig SinkConfig,
	sourceConfig SourceConfig,
	ethConn *ethereum.Connection
	subConn *relaychain.Connection
	beefyLightClient beefylightclient.Contract
}

func NewNaka(
	sinkConfig SinkConfig,
	sourceConfig sourceConfig,
	ethConn *ethereum.Connection,
	subConn *relaychain.Connection,
) *EthereumWriter {
	return &Naka{
		sinkConfig, sourceConfig, ethConn, subConn,
	}
}

func (n *Naka) Start(ctx context.Context, eg errgroup.Group) error {
	address := common.HexToAddress(n.sinkConfig.Contracts.BeefyLightClient)
	contract, err := beefylightclient.NewContract(address, n.ethConn.GetClient())
	if err != nil {
		return 0, err
	}
	wr.beefyLightClient = contract

	b, err := n.beefyLightClient.LatestBeefyBlock(nil)
	if err != nil {
		return fmt.Errorf("fetch latestBeefyBlock: %w", err)
	}

	fo, err := n.beefyLightClient.NextValidatorSet(nil)
	if err != nil {
		return fmt.Errorf("fetch nextValidatorSet: %w", err)
	}


	eg.Go(func() error {
		err := n.watchNewSessionEvents(ctx)
		log.WithField("reason", err).Info("Shutting down NewSession event watcher")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})
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

			n.processMessage(ctx, ev)
		}
	}
}

func (n *Naka) processMessage(ctx context.Context, ev *beefylightclient.ContractNewSession) error {

	// we can use any block except the latest beefy block
	blockToProve := ev.BlockNumber - 1
	response, err := n.subConn.GenerateProofForBlock(blockToProve, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		return fmt.Errorf("proof generation for %v: %w", blockToProve, err)
	}

	proof, err := merkle.ConvertToSimplifiedMMRProof(response.BlockHash, uint64(response.Proof.LeafIndex),
		response.Leaf, uint64(response.Proof.LeafCount), response.Proof.Items)
	if err != nil {
		return fmt.Errorf("simplified proof conversion for block %v: %w", blockToProve, err)
	}

	inputLeaf := beefylightclient.BeefyLightClientMMRLeaf{
		Version:              uint8(proof.Leaf.Version),
		ParentNumber:         uint32(proof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           proof.Leaf.ParentNumberAndHash.Hash,
		ParachainHeadsRoot:   proof.Leaf.ParachainHeads,
		NextAuthoritySetId:   uint64(proof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(proof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: proof.Leaf.BeefyNextAuthoritySet.Root,
	}

	merkleProofItems := [][32]byte{}
	for _, mmrProofItem := range proof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, mmrProofItem)
	}

	inputProof := beefylightclient.SimplifiedMMRProof{
		MerkleProofItems:         merkleProofItems,
		MerkleProofOrderBitField: proof.MerkleProofOrder,
	}

	opts := bind.TransactOpts{
		Context: ctx,
	}

	tx, err := n.beefyLightClient.UpdateValidatorSet(&opts, inputLeaf, inputProof)
	if err != nil {
		return fmt.Errorf("transaction UpdateValidatorSet: %w", err)
	}

	return nil
}
