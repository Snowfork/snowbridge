package cmd

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
	"github.com/spf13/cobra"
)

const PARACHAIN_ID = 200

func subBeefyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "sub-beefy",
		Short:   "Subscribe beefy messages",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay sub-beefy",
		RunE:    SubBeefyFn,
	}
	return cmd
}

func SubBeefyFn(cmd *cobra.Command, _ []string) error {
	subBeefyJustifications(cmd.Context())
	return nil
}

func subBeefyJustifications(ctx context.Context) error {
	log.Info("Loading config")
	config, err := core.LoadConfig()
	if err != nil {
		log.Error(err)
		return err
	}

	log := log.WithField("script", "beefy")

	relaychainEndpoint := config.Relaychain.Endpoint
	relaychainConn := relaychain.NewConnection(relaychainEndpoint, log)
	err = relaychainConn.Connect(ctx)
	if err != nil {
		log.Error(err)
		return err
	}

	parachainEndpoint := config.Parachain.Endpoint
	parachainConn := parachain.NewConnection(parachainEndpoint, nil, log)
	err = parachainConn.Connect(ctx)
	if err != nil {
		log.Error(err)
		return err
	}

	ch := make(chan interface{})

	log.Info("Subscribing to beefy justifications")
	sub, err := relaychainConn.GetAPI().Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case msg := <-ch:

			signedCommitment := &store.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				log.WithError(err).Error("Failed to decode BEEFY commitment messages")
			}

			blockNumber := signedCommitment.Commitment.BlockNumber

			log.WithField("commitmentBlockNumber", blockNumber).Info("Witnessed a new BEEFY commitment: \n")
			if len(signedCommitment.Signatures) == 0 {
				log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}
			log.WithField("blockNumber", blockNumber+1).Info("Getting hash for next block")
			nextBlockHash, err := relaychainConn.GetAPI().RPC.Chain.GetBlockHash(uint64(blockNumber + 1))
			if err != nil {
				log.WithError(err).Error("Failed to get block hash")
			}
			log.WithField("blockHash", nextBlockHash.Hex()).Info("Got blockhash")
			GetMMRLeafForBlock(uint64(blockNumber), nextBlockHash, relaychainConn)
			header, err := GetParaheads(nextBlockHash, relaychainConn)
			if err != nil {
				log.WithError(err).Error("Failed to get para heads")
				return err
			}
			GetParaHeadData(header, parachainConn)
		}
	}
}

func GetParaheads(blockHash types.Hash, relaychainConn *relaychain.Connection) (string, error) {
	type ParaId types.U32

	type ParaHeadsQuery struct {
		ParaId ParaId
	}

	query := ParaHeadsQuery{ParaId: ParaId(types.NewU32(200))}

	encoded, err := types.EncodeToBytes(query)
	if err != nil {
		log.WithError(err).Error("Error")
	}

	allParaHeadsStorageKey, err := types.CreateStorageKey(
		relaychainConn.GetMetadata(),
		"Paras",
		"Heads", encoded, nil)
	if err != nil {
		log.WithError(err).Error("Failed to create parachain header storage key")
	}

	// TODO1 - in polkadotjs webapp it seems possible to query this state via paras.heads without specifying any option
	// and get all parachain heads as a response. eg: trying it on Rococo returns a lot of paraheads.
	// I think we'll need this (to be able to check against the parachain headers hash
	// of all headers in MMRLeaf.ParachainHeads), so we should get this working too.
	response, err := relaychainConn.GetAPI().RPC.State.GetStorageRaw(allParaHeadsStorageKey, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to get all parachain headers")
	}

	// TODO2 - the above query returns some extra bytes, related the the HeadData type (try this state query in polkadotjs
	// webapp for example). These extra bytes I think are the parachain ID, so the response type needs to account for
	// this properly. the below is just a hack to get the actual header out. It's also not clear to me if the response
	// contains the entire header, or just a hash of the header, or some truncated header? If it's the entire header,
	// then great we can use it entirely instead of querying for it in a follow up call
	header := response.Hex()
	actualHeader := fmt.Sprintf("%s%s", "0x", header[6:70])
	log.WithField("actualHeader", actualHeader).Info("Sliced header response into actual header")

	log.WithField("header", actualHeader).Info("Got parachain header")
	return actualHeader, nil
}

func GetParaHeadData(header string, parachainConn *parachain.Connection) {
	headerHash, err := types.NewHashFromHexString(header)
	if err != nil {
		log.WithError(err).Error("Failed to create header hash")
	}

	log.WithFields(logrus.Fields{
		"hash": headerHash.Hex(),
	}).Info("Querying for parachain header")
	headerData, err := parachainConn.Api().RPC.Chain.GetHeader(headerHash)
	if err != nil {
		log.WithError(err).Error("Failed to get parachain header")
	}

	log.WithFields(logrus.Fields{
		"headerData": headerData,
	}).Info("Retrieved full parachain header")

	// TODO4 - Get the header digest and commitment logs from this particular header once above is working.
	// const headerLogs = headData.toJSON().digest && headData.toJSON().digest.logs;
	// const commitmentLog = headerLogs && headerLogs[0];
	// if (commitmentLog) {
	//   console.log("Got new commitment: ");
	//   console.log({ commitmentLog });
	// }

	// TODO5 - Get the messages in that commitments

	// TODO6 - Update all above code to make sure to check all new parachain blocks that have been added to the MMR
	// when there is a new beefy justification, not just the newest parachain block in the MMR
}

func GetMMRLeafForBlock(blockNumber uint64, blockHash types.Hash, relaychainConn *relaychain.Connection) {
	log.WithFields(logrus.Fields{
		"blockNumber": blockNumber,
		"blockHash":   blockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")
	proofResponse, err := relaychainConn.GetAPI().RPC.MMR.GenerateProof(blockNumber, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to generate mmr proof")
	}

	log.WithFields(logrus.Fields{
		"BlockHash":                       proofResponse.BlockHash.Hex(),
		"Leaf.ParentNumber":               proofResponse.Leaf.ParentNumberAndHash.ParentNumber,
		"Leaf.Hash":                       proofResponse.Leaf.ParentNumberAndHash.Hash.Hex(),
		"Leaf.ParachainHeads":             proofResponse.Leaf.ParachainHeads.Hex(),
		"Leaf.BeefyNextAuthoritySet.ID":   proofResponse.Leaf.BeefyNextAuthoritySet.ID,
		"Leaf.BeefyNextAuthoritySet.Len":  proofResponse.Leaf.BeefyNextAuthoritySet.Len,
		"Leaf.BeefyNextAuthoritySet.Root": proofResponse.Leaf.BeefyNextAuthoritySet.Root.Hex(),
		"Proof.LeafIndex":                 proofResponse.Proof.LeafIndex,
		"Proof.LeafCount":                 proofResponse.Proof.LeafCount,
	}).Info("Generated MMR Proof")
}
