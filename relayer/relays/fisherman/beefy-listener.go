package fisherman

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	ethereumConn        *ethereum.Connection
	beefyClientContract *contracts.BeefyClient
	relaychainConn      *relaychain.Connection
}

func NewBeefyListener(
	config *SourceConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
) *BeefyListener {
	return &BeefyListener{
		config:         config,
		ethereumConn:   ethereumConn,
		relaychainConn: relaychainConn,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {
	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyClient)
	beefyClientContract, err := contracts.NewBeefyClient(address, li.ethereumConn.Client())
	if err != nil {
		return err
	}
	li.beefyClientContract = beefyClientContract

	err = li.subscribeNewBEEFYEvents(ctx)
	if err != nil {
		if errors.Is(err, context.Canceled) {
			return nil
		}
		return err
	}

	return nil
}

func (li *BeefyListener) subscribeNewBEEFYEvents(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)

	sub, err := li.ethereumConn.Client().SubscribeNewHead(ctx, headers)
	if err != nil {
		return fmt.Errorf("creating ethereum header subscription: %w", err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			return fmt.Errorf("header subscription: %w", err)
		case gethheader := <-headers:
			blockNumber := gethheader.Number.Uint64()
			err := li.checkSubmitInitialEquivocation(ctx, blockNumber)
			if err != nil {
				return fmt.Errorf("report submit initial equivocation %v: %w", blockNumber, err)
			}
			err = li.checkSubmitFinalEquivocation(ctx, blockNumber)
			if err != nil {
				return fmt.Errorf("report submit final equivocation %v: %w", blockNumber, err)
			}
		}
	}
}

// handles checking submitInitial and reporting of equivocations for a given block number
func (li *BeefyListener) checkSubmitInitialEquivocation(ctx context.Context, blockNumber uint64) error {

	latestBeefyHash, _, err := li.getLatestBeefyBlock()
	if err != nil {
		return fmt.Errorf("get latest Beefy block: %w", err)
	}

	// Check NewTicket events for equivocation
	contractNewTicketEvents, err := li.queryNewTicketEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		return fmt.Errorf("query NewTicket event logs in block %v: %w", blockNumber, err)
	}

	if len(contractNewTicketEvents) == 0 {
		return nil
	}

	log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewTicket events in block %d", len(contractNewTicketEvents), blockNumber))
	// Iterate over all NewTicket events in the block
	for _, event := range contractNewTicketEvents {
		commitment, validatorProof, err := li.parseSubmitInitial(ctx, event.Raw)
		err = li.reportFutureBlockEquivocation(ctx, commitment, validatorProof, latestBeefyHash)
		if err != nil {
			return fmt.Errorf("report future block equivocation: %w", err)
		}
		err = li.reportForkEquivocation(ctx, commitment, validatorProof, latestBeefyHash)
		if err != nil {
			return fmt.Errorf("report fork equivocation: %w", err)
		}
	}
	return nil
}

// handles checking submitFinal and reporting of equivocations for a given block number
func (li *BeefyListener) checkSubmitFinalEquivocation(ctx context.Context, blockNumber uint64) error {
	latestBeefyHash, _, err := li.getLatestBeefyBlock()
	if err != nil {
		return fmt.Errorf("get latest Beefy block: %w", err)
	}

	// Also check NewMMRRoot events for equivocation
	contractNewMMRRootEvents, err := li.queryNewMMRRootEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		return fmt.Errorf("query NewMMRRoot event logs in block %v: %w", blockNumber, err)
	}

	if len(contractNewMMRRootEvents) == 0 {
		return nil
	}
	log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewMMRRoot events in block %d", len(contractNewMMRRootEvents), blockNumber))
	// Iterate over all NewMMRRoot events in the block
	for _, event := range contractNewMMRRootEvents {
		commitment, validatorProofs, err := li.parseSubmitFinal(ctx, event.Raw)
		for _, validatorProof := range validatorProofs {
			err = li.reportFutureBlockEquivocation(ctx, commitment, validatorProof, latestBeefyHash)
			if err != nil {
				return fmt.Errorf("report future block equivocation: %w", err)
			}
			err = li.reportForkEquivocation(ctx, commitment, validatorProof, latestBeefyHash)
			if err != nil {
				return fmt.Errorf("report fork equivocation: %w", err)
			}
		}
	}
	return nil
}

// decodes the commitment in submitInitial or submitFinal call
func (li *BeefyListener) parseCommitment(callData []byte) (contracts.BeefyClientCommitment, error) {
	// decode the callData
	methodName, decoded, err := decodeTransactionCallData(callData)
	if err != nil {
		return contracts.BeefyClientCommitment{}, err
	}
	if methodName != "submitInitial" && methodName != "submitFinal" {
		return contracts.BeefyClientCommitment{}, fmt.Errorf("unexpected method name: %s", methodName)
	}

	log.WithFields(log.Fields{
		"raw commitment": decoded["commitment"],
		"method":         methodName,
	}).Debug("Decoded transaction call data")

	// Extract the commitment
	commitmentRaw := decoded["commitment"].(struct {
		BlockNumber    uint32 `json:"blockNumber"`
		ValidatorSetID uint64 `json:"validatorSetID"`
		Payload        []struct {
			PayloadID [2]uint8 `json:"payloadID"`
			Data      []uint8  `json:"data"`
		} `json:"payload"`
	})

	commitment := contracts.BeefyClientCommitment{
		BlockNumber:    commitmentRaw.BlockNumber,
		ValidatorSetID: commitmentRaw.ValidatorSetID,
		Payload:        make([]contracts.BeefyClientPayloadItem, len(commitmentRaw.Payload)),
	}

	// Convert payload items
	for i, p := range commitmentRaw.Payload {
		commitment.Payload[i] = contracts.BeefyClientPayloadItem{
			PayloadID: p.PayloadID,
			Data:      p.Data,
		}
	}

	return commitment, nil
}

// decodes the remainder of a submitInitial call - only needed if equivocation detected
func (li *BeefyListener) parseSubmitInitialProof(callData []byte) (contracts.BeefyClientValidatorProof, error) {
	// decode the callData
	methodName, decoded, err := decodeTransactionCallData(callData)
	if err != nil {
		return contracts.BeefyClientValidatorProof{}, err
	}
	if methodName != "submitInitial" {
		return contracts.BeefyClientValidatorProof{}, fmt.Errorf("unexpected method name: %s", methodName)
	}
	// bitfield := decoded["bitfield"].([]*big.Int)
	log.WithFields(log.Fields{
		// "raw commitment": decoded["commitment"],
		"raw proof": decoded["proof"],
	}).Debug("Decoded proof from submitInitial")

	// Extract validator proof
	proofRaw := decoded["proof"].(struct {
		V       uint8          `json:"v"`
		R       [32]byte       `json:"r"`
		S       [32]byte       `json:"s"`
		Index   *big.Int       `json:"index"`
		Account common.Address `json:"account"`
		Proof   [][32]byte     `json:"proof"`
	})

	proof := contracts.BeefyClientValidatorProof{
		V:       proofRaw.V,
		R:       proofRaw.R,
		S:       proofRaw.S,
		Index:   proofRaw.Index,
		Account: proofRaw.Account,
		Proof:   proofRaw.Proof,
	}

	return proof, nil
}

// decodes the remainder of a submitFinal call - only needed if equivocation detected
func (li *BeefyListener) parseSubmitFinalProofs(callData []byte) ([]contracts.BeefyClientValidatorProof, error) {
	// decode the callData
	methodName, decoded, err := decodeTransactionCallData(callData)
	if err != nil {
		return nil, fmt.Errorf("decodeTransactionCallData: %w", err)
	}
	if methodName != "submitFinal" {
		return nil, fmt.Errorf("unexpected method name: %s", methodName)
	}

	log.WithFields(log.Fields{
		// "raw commitment": decoded["commitment"],
		"raw proofs": decoded["proofs"],
	}).Debug("Decoded proofs from submitFinal")

	// Extract validator proof
	proofsRaw := decoded["proofs"].([]struct {
		V       uint8          `json:"v"`
		R       [32]byte       `json:"r"`
		S       [32]byte       `json:"s"`
		Index   *big.Int       `json:"index"`
		Account common.Address `json:"account"`
		Proof   [][32]byte     `json:"proof"`
	})

	proofs := make([]contracts.BeefyClientValidatorProof, len(proofsRaw))

	for i, pr := range proofsRaw {

		proof := contracts.BeefyClientValidatorProof{
			V:       pr.V,
			R:       pr.R,
			S:       pr.S,
			Index:   pr.Index,
			Account: pr.Account,
			Proof:   pr.Proof,
		}
		proofs[i] = proof
	}

	return proofs, nil
}

// queryNewMMRRootEvents queries NewMMRRoot events from the BeefyClient contract
func (li *BeefyListener) queryNewMMRRootEvents(
	ctx context.Context, start uint64,
	end *uint64,
) ([]*contracts.BeefyClientNewMMRRoot, error) {
	var events []*contracts.BeefyClientNewMMRRoot
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyClientContract.FilterNewMMRRoot(&filterOps)
	if err != nil {
		return nil, err
	}

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return nil, err
			}
			break
		}
		events = append(events, iter.Event)
	}

	return events, nil
}

// queryNewTicketEvents queries NewTicket events from the BeefyClient contract
func (li *BeefyListener) queryNewTicketEvents(
	ctx context.Context, start uint64,
	end *uint64,
) ([]*contracts.BeefyClientNewTicket, error) {
	var events []*contracts.BeefyClientNewTicket
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}
	iter, err := li.beefyClientContract.FilterNewTicket(&filterOps)
	if err != nil {
		return nil, err
	}

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return nil, err
			}
			break
		}
		events = append(events, iter.Event)
	}

	return events, nil
}

// handles reporting of future block equivocations
func (li *BeefyListener) reportFutureBlockEquivocation(ctx context.Context, commitment contracts.BeefyClientCommitment, validatorProof contracts.BeefyClientValidatorProof, latestHash types.Hash) error {
	latestBlock, err := li.relaychainConn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return fmt.Errorf("get block: %w", err)
	}
	latestBlockNumber := uint64(latestBlock.Block.Header.Number)

	if uint64(commitment.BlockNumber) > uint64(latestBlockNumber) {
		// Future block equivocation handling
		log.WithFields(log.Fields{
			"commitment.payload.data": fmt.Sprintf("%#x", commitment.Payload[0].Data),
			"latestBlock":             latestBlockNumber,
		}).Warning("Detected submitInitial for future block")

		meta, err := li.relaychainConn.API().RPC.State.GetMetadataLatest()
		if err != nil {
			return fmt.Errorf("get metadata: %w", err)
		}

		extrinsicName := "Beefy.report_future_block_voting"
		// call: c805
		// build vote payload for equivocation proof
		offenderPubKeyCompressed, offenderSig, err := getOffenderPubKeyAndSig(commitment, validatorProof)
		if err != nil {
			return fmt.Errorf("get offender pubkey and sig: %w", err)
		}

		payload1 := constructVotePayload(commitment, offenderPubKeyCompressed, offenderSig)

		// keyOwnership Proof
		keyOwnershipProof, err := li.getKeyOwnershipProof(meta, latestHash, latestBlockNumber, offenderPubKeyCompressed, commitment.ValidatorSetID)
		if err != nil {
			return fmt.Errorf("get key ownership proof: %w", err)
		}

		payload2 := keyOwnershipProof[3:]
		// payload := []interface{}{types.NewBytes(payload1), types.NewBytes(payload2)}
		// combine payload1 and payload2
		payload := []interface{}{types.NewData(append(payload1, payload2...))}
		c, err := types.NewCall(meta, extrinsicName, payload...)
		// c, err := types.NewCall(meta, extrinsicName, types.NewBytes(payload1), types.NewBytes(payload2))
		if err != nil {
			return fmt.Errorf("create call: %w", err)
		}

		ext, err := li.signedExtrinsicFromCall(meta, c)

		// Send the extrinsic
		sub, err := li.relaychainConn.API().RPC.Author.SubmitAndWatchExtrinsic(ext)
		if err != nil {
			return fmt.Errorf("submit and watch extrinsic: %w", err)
		}
		err = li.watchExtrinsicSubscription(sub)
		if err != nil {
			return fmt.Errorf("Extrinsic submission failed: %w", err)
		}
		log.Info("Future block equivocation report complete")
	}

	return nil
}

// handles reporting of fork equivocations
func (li *BeefyListener) reportForkEquivocation(ctx context.Context, commitment contracts.BeefyClientCommitment, validatorProof contracts.BeefyClientValidatorProof, latestHash types.Hash) error {
	latestBlock, err := li.relaychainConn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return fmt.Errorf("get block: %w", err)
	}
	latestBlockNumber := uint64(latestBlock.Block.Header.Number)

	canonicalBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(commitment.BlockNumber))
	if err != nil {
		return fmt.Errorf("fetch block hash: %w", err)
	}
	canonicalMmrRootHash, err := li.relaychainConn.GetMMRRootHash(canonicalBlockHash)

	if err != nil {
		return fmt.Errorf("retrieve MMR root hash at block %v: %w", canonicalBlockHash.Hex(), err)
	}

	if canonicalMmrRootHash != types.NewHash(commitment.Payload[0].Data) {
		// Fork equivocation handling
		log.WithFields(log.Fields{
			"commitment":               commitment,
			"commitment.payload.data":  fmt.Sprintf("%#x", commitment.Payload[0].Data),
			"proof":                    validatorProof,
			"correspondingMMRRootHash": canonicalMmrRootHash,
		}).Warning("MMR root hash does NOT match the commitment payload")

		meta, err := li.relaychainConn.API().RPC.State.GetMetadataLatest()
		if err != nil {
			return fmt.Errorf("get metadata: %w", err)
		}

		extrinsicName := "Beefy.report_fork_voting"
		// call: c803
		// build vote payload for equivocation proof
		offenderPubKeyCompressed, offenderSig, err := getOffenderPubKeyAndSig(commitment, validatorProof)
		if err != nil {
			return fmt.Errorf("get offender pubkey and sig: %w", err)
		}

		payload1 := constructVotePayload(commitment, offenderPubKeyCompressed, offenderSig)

		// Ancestry Proof
		payload2, err := li.constructAncestryProofPayload(commitment, latestHash)
		if err != nil {
			return fmt.Errorf("build ancestry proof payload: %w", err)
		}

		// Header
		payload3, err := types.EncodeToBytes(latestBlock.Block.Header)

		// Key Ownership Proof
		keyOwnershipProof, err := li.getKeyOwnershipProof(meta, latestHash, latestBlockNumber, offenderPubKeyCompressed, commitment.ValidatorSetID)
		if err != nil {
			return fmt.Errorf("get key ownership proof: %w", err)
		}

		payload4 := keyOwnershipProof[3:]
		// payload := []interface{}{types.NewBytes(payload1), types.NewBytes(payload2)}
		// combine payload1 and payload2
		payloadEquivocationProof := append(payload1, payload2...)
		payloadEquivocationProof = append(payloadEquivocationProof, payload3...)
		payload := []interface{}{types.NewData(append(payloadEquivocationProof, payload4...))}
		c, err := types.NewCall(meta, extrinsicName, payload...)
		// c, err := types.NewCall(meta, extrinsicName, types.NewBytes(payload1), types.NewBytes(payload2))
		if err != nil {
			return fmt.Errorf("create call: %w", err)
		}

		ext, err := li.signedExtrinsicFromCall(meta, c)

		// Send the extrinsic
		sub, err := li.relaychainConn.API().RPC.Author.SubmitAndWatchExtrinsic(ext)
		if err != nil {
			return fmt.Errorf("submit and watch extrinsic: %w", err)
		}
		err = li.watchExtrinsicSubscription(sub)
		if err != nil {
			return fmt.Errorf("Extrinsic submission failed: %w", err)
		}
		log.Info("Fork equivocation report complete")
	}
	return nil
}

// decodes the submitInitial call
func (li *BeefyListener) parseSubmitInitial(ctx context.Context, eventLog gethTypes.Log) (commitment contracts.BeefyClientCommitment, validatorProof contracts.BeefyClientValidatorProof, err error) {
	callData, err := li.getTransactionCallData(ctx, eventLog.TxHash)
	if err != nil {
		return commitment, validatorProof, fmt.Errorf("get transaction call data: %w", err)
	}

	commitment, err = li.parseCommitment(callData)
	if err != nil {
		return commitment, validatorProof, fmt.Errorf("parse submit initial commitment: %w", err)
	}

	validatorProof, err = li.parseSubmitInitialProof(callData)
	if err != nil {
		return commitment, validatorProof, fmt.Errorf("get transaction call data: %w", err)
	}
	return commitment, validatorProof, nil
}

// decodes the submitFinal call
func (li *BeefyListener) parseSubmitFinal(ctx context.Context, eventLog gethTypes.Log) (commitment contracts.BeefyClientCommitment, validatorProofs []contracts.BeefyClientValidatorProof, err error) {
	callData, err := li.getTransactionCallData(ctx, eventLog.TxHash)
	if err != nil {
		return commitment, validatorProofs, fmt.Errorf("get transaction call data: %w", err)
	}

	commitment, err = li.parseCommitment(callData)
	if err != nil {
		return commitment, validatorProofs, fmt.Errorf("parse submit initial commitment: %w", err)
	}

	validatorProofs, err = li.parseSubmitFinalProofs(callData)
	if err != nil {
		return commitment, validatorProofs, fmt.Errorf("get transaction call data: %w", err)
	}
	return commitment, validatorProofs, nil
}
