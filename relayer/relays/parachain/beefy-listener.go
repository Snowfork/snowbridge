package parachain

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/ofac"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	scheduleConfig      *ScheduleConfig
	ethereumConn        *ethereum.Connection
	beefyClientContract *contracts.BeefyClient
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	ofac                *ofac.OFAC
	paraID              uint32
	tasks               chan<- *Task
	scanner             *Scanner
}

func NewBeefyListener(
	config *SourceConfig,
	scheduleConfig *ScheduleConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	ofac *ofac.OFAC,
	tasks chan<- *Task,
) *BeefyListener {
	return &BeefyListener{
		config:              config,
		scheduleConfig:      scheduleConfig,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		ofac:                ofac,
		tasks:               tasks,
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

	// fetch ParaId
	paraIDKey, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "ParachainInfo", "ParachainId", nil, nil)
	if err != nil {
		return err
	}
	var paraID uint32
	ok, err := li.parachainConnection.API().RPC.State.GetStorageLatest(paraIDKey, &paraID)
	if err != nil {
		return fmt.Errorf("fetch parachain id: %w", err)
	}
	if !ok {
		return fmt.Errorf("parachain id missing")
	}
	li.paraID = paraID

	li.scanner = &Scanner{
		config:    li.config,
		ethConn:   li.ethereumConn,
		relayConn: li.relaychainConn,
		paraConn:  li.parachainConnection,
		paraID:    paraID,
		ofac:      li.ofac,
	}

	eg.Go(func() error {
		defer close(li.tasks)

		// Subscribe NewMMRRoot event logs and fetch parachain message commitments
		// since latest beefy block
		beefyBlockNumber, _, err := li.fetchLatestBeefyBlock(ctx)
		if err != nil {
			return fmt.Errorf("fetch latest beefy block: %w", err)
		}

		err = li.doScan(ctx, beefyBlockNumber)
		if err != nil {
			return fmt.Errorf("scan for sync tasks bounded by BEEFY block %v: %w", beefyBlockNumber, err)
		}

		err = li.subscribeNewBEEFYEvents(ctx)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}

		return nil
	})

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
			// TODO: for slashing, we may want to
			// 1. scan older blocks as well if the latest BEEFY commitment stored on Ethereum doesn't match the commitment at that block # on the relay chain
			// 2. potentially scan older blocks even if latest BEEFY commitment is sound (honest relayers may have saved the day, but the adversaries should still be slashed)
			// 3. scan older tickets as well, since they may have optimistically tried their luck on the subsampling without a `submitFinal` call
			// in all cases, this scan should not go past the slashability horizon, i.e. only be performed where the validators are still bonded
			// this may also have to query the relay chain for any reported equivocations
			blockNumber := gethheader.Number.Uint64()
			contractNewMMRRootEvents, err := li.queryNewMMRRootEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				return fmt.Errorf("query NewMMRRoot event logs in block %v: %w", blockNumber, err)
			}

			if len(contractNewMMRRootEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewMMRRoot events in block %d", len(contractNewMMRRootEvents), blockNumber))
				// Only process the last emitted event in the block
				event := contractNewMMRRootEvents[len(contractNewMMRRootEvents)-1]
				log.WithFields(log.Fields{
					"beefyBlockNumber":    event.BlockNumber,
					"ethereumBlockNumber": event.Raw.BlockNumber,
					"ethereumTxHash":      event.Raw.TxHash.Hex(),
				}).Info("Witnessed a new MMRRoot event")

				err = li.doScan(ctx, event.BlockNumber)
				if err != nil {
					return fmt.Errorf("scan for sync tasks bounded by BEEFY block %v: %w", event.BlockNumber, err)
				}
			}

			contractNewTicketEvents, err := li.queryNewTicketEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				return fmt.Errorf("query NewTicket event logs in block %v: %w", blockNumber, err)
			}

			if len(contractNewTicketEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient.NewTicket events in block %d", len(contractNewTicketEvents), blockNumber))
				// Iterate over all NewTicket events in the block
				for _, event := range contractNewTicketEvents {

					callData, err := li.getTransactionCallData(ctx, event.Raw.TxHash)
					if err != nil {
						return fmt.Errorf("get transaction call data: %w", err)
					}

					log.WithFields(log.Fields{
						"beefyBlockNumber":    event.BlockNumber,
						"ethereumBlockNumber": event.Raw.BlockNumber,
						"ethereumTxHash":      event.Raw.TxHash.Hex(),
						"ethereumTxIndex":     event.Raw.TxHash.Hex(),
						"rawEvent":            event.Raw,
					}).Info("Witnessed a new Ticket event")

					commitment, bitfield, validatorProof, err := li.parseSubmitInitial(callData)
					if err != nil {
						log.WithError(err).Warning("Failed to decode transaction call data")
					}
					// TODO: handle tickets submitted for future blocks
					latestHash, latestBlock, err := li.getLatestBlockInfo()
					if err != nil {
						return fmt.Errorf("get latest block info: %w", err)
					}
					latestBlockNumber := uint64(latestBlock.Block.Header.Number)

					if event.BlockNumber > uint64(latestBlockNumber) {
						// Future block equivocation handling
						log.WithFields(log.Fields{
							"commitment.payload.data": fmt.Sprintf("%#x", commitment.Payload[0].Data),
							"proof":                   validatorProof,
							"latestBlock":             latestBlockNumber,
						}).Warning("Detected submitInitial for future block")

						log.Info("schedule ID", li.scheduleConfig.ID)
						if li.scheduleConfig.ID != 0 {
							log.Info("testing: only submitting from relayer 0 - skipping")
							return nil
						}

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
						log.Info("calling api")

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
						// res, err := li.relaychainConn.API().RPC.Author.SubmitExtrinsic(ext)
						if err != nil {
							log.Error("Failed to submit extrinsic: ", err, sub)
						} else {
							err := li.watchExtrinsicSubscription(sub)
							log.Info("Extrinsic submitted: ", sub)
							if err != nil {
								log.Error("Extrinsic submission failed: ", err)
							} else {
								log.Info("equivocation report complete")
							}
						}

					} else {
						canonicalBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(event.BlockNumber)
						if err != nil {
							return fmt.Errorf("fetch block hash: %w", err)
						}
						canonicalMmrRootHash, err := li.relaychainConn.GetMMRRootHash(canonicalBlockHash)
						if err != nil {
							return fmt.Errorf("retrieve MMR root hash at block %v: %w", canonicalBlockHash.Hex(), err)
						} else {
							log.WithFields(log.Fields{
								"commitment":               commitment,
								"bitfield":                 bitfield,
								"commitment.payload.data":  fmt.Sprintf("%#x", commitment.Payload[0].Data),
								"proof":                    validatorProof,
								"correspondingMMRRootHash": canonicalMmrRootHash,
							}).Info("Decoded transaction call data for NewTicket event")
							if canonicalMmrRootHash != types.NewHash(commitment.Payload[0].Data) {
								log.WithFields(log.Fields{
									"commitment.payload.data":  fmt.Sprintf("%#x", commitment.Payload[0].Data),
									"proof":                    validatorProof,
									"correspondingMMRRootHash": canonicalMmrRootHash,
								}).Warning("MMR root hash does NOT match the commitment payload")

								if li.scheduleConfig.ID != 0 {
									log.Info("testing: only submitting from relayer 0 - skipping")
									return nil
								}

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
								payload2, err := li.constructAncestryProofPayload(commitment)
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
									log.Error("Failed to submit extrinsic: ", err, sub)
								} else {
									err := li.watchExtrinsicSubscription(sub)
									if err != nil {
										log.Error("Extrinsic submission failed: ", err)
									} else {
										log.Info("equivocation report complete")
									}
								}

							} else {
								log.Debug("MMR root hash DOES match the commitment payload")
							}
						}
					}
					// TODO: should this also invoke a scan? I reckon not, since the scan's purpose in my understanding is to check which parachain messages have to be relayed (stale or new), and an update to the MMR root would necessitate creating proofs against the new MMR root, rather than the old one.
				}
			}
		}
	}
}

func (li *BeefyListener) getTransactionCallData(ctx context.Context, txHash common.Hash) ([]byte, error) {
	// Get the transaction
	tx, _, err := li.ethereumConn.Client().TransactionByHash(ctx, txHash)
	if err != nil {
		return nil, fmt.Errorf("get transaction: %w", err)
	}

	// Get the input data
	return tx.Data(), nil
}

func (li *BeefyListener) decodeTransactionCallData(callData []byte) (string, map[string]interface{}, error) {
	// Parse the ABI
	parsedABI, err := abi.JSON(strings.NewReader(contracts.BeefyClientMetaData.ABI))
	if err != nil {
		return "", nil, fmt.Errorf("parse ABI: %w", err)
	}

	// Get the method signature from the first 4 bytes
	methodSig := callData[:4]
	method, err := parsedABI.MethodById(methodSig)
	if err != nil {
		return "", nil, fmt.Errorf("get method from signature: %w", err)
	}

	// Decode the parameters
	params, err := method.Inputs.Unpack(callData[4:])
	if err != nil {
		return "", nil, fmt.Errorf("unpack parameters: %w", err)
	}

	// Convert to map for handling
	decoded := make(map[string]interface{})
	for i, param := range params {
		log.WithFields(log.Fields{
			"name":      method.Inputs[i].Name,
			"param":     param,
			"param raw": param,
		}).Info("Decoded transaction call data for NewTicket event")
		decoded[method.Inputs[i].Name] = param
	}

	return method.Name, decoded, nil
}

// decodes a submitInitial call
// TODO: not necessary to parse everything - doing it for now in case something ends up needed & avoid losing time on not finding it.
// strictly speaking only want to parse the commitment initially to get blocknumber & payload, and then if those do not match the relay chain's canonical payload, parse the validator proof to extract signature and validator who signed it
func (li *BeefyListener) parseSubmitInitial(callData []byte) (contracts.BeefyClientCommitment, []*big.Int, contracts.BeefyClientValidatorProof, error) {
	// decode the callData
	methodName, decoded, err := li.decodeTransactionCallData(callData)
	if err != nil {
		return contracts.BeefyClientCommitment{}, nil, contracts.BeefyClientValidatorProof{}, err
	}
	if methodName != "submitInitial" {
		return contracts.BeefyClientCommitment{}, nil, contracts.BeefyClientValidatorProof{}, fmt.Errorf("unexpected method name: %s", methodName)
	}

	log.WithFields(log.Fields{
		"raw commitment": decoded["commitment"],
		"raw proof":      decoded["proof"],
	}).Info("Decoded transaction call data for NewTicket event")

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

	bitfield := decoded["bitfield"].([]*big.Int)

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

	return commitment, bitfield, proof, nil
}

func (li *BeefyListener) doScan(ctx context.Context, beefyBlockNumber uint64) error {
	tasks, err := li.scanner.Scan(ctx, beefyBlockNumber)
	if err != nil {
		return err
	}
	for _, task := range tasks {
		paraNonce := (*task.MessageProofs)[0].Message.Nonce
		waitingPeriod := (uint64(paraNonce) + li.scheduleConfig.TotalRelayerCount - li.scheduleConfig.ID) % li.scheduleConfig.TotalRelayerCount
		err = li.waitAndSend(ctx, task, waitingPeriod)
		if err != nil {
			return fmt.Errorf("wait task for nonce %d: %w", paraNonce, err)
		}
	}

	return nil
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

// Fetch the latest verified beefy block number and hash from Ethereum
func (li *BeefyListener) fetchLatestBeefyBlock(ctx context.Context) (uint64, types.Hash, error) {
	number, err := li.beefyClientContract.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		return 0, types.Hash{}, fmt.Errorf("fetch latest beefy block from light client: %w", err)
	}

	hash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(number)
	if err != nil {
		return 0, types.Hash{}, fmt.Errorf("fetch block hash: %w", err)
	}

	return number, hash, nil
}

// The maximum paras that will be included in the proof.
// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/parachains/src/paras/mod.rs#L1225-L1232
const MaxParaHeads = 1024

// Generates a proof for an MMR leaf, and then generates a merkle proof for our parachain header, which should be verifiable against the
// parachains root in the mmr leaf.
func (li *BeefyListener) generateProof(ctx context.Context, input *ProofInput, header *types.Header) (*ProofOutput, error) {
	latestBeefyBlockNumber, latestBeefyBlockHash, err := li.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return nil, fmt.Errorf("fetch latest beefy block: %w", err)
	}

	log.WithFields(log.Fields{
		"beefyBlock": latestBeefyBlockNumber,
		"leafIndex":  input.RelayBlockNumber,
	}).Info("Generating MMR proof")

	// Generate the MMR proof for the polkadot block.
	mmrProof, err := li.relaychainConn.GenerateProofForBlock(
		input.RelayBlockNumber+1,
		latestBeefyBlockHash,
	)
	if err != nil {
		return nil, fmt.Errorf("generate MMR leaf proof: %w", err)
	}

	simplifiedProof, err := merkle.ConvertToSimplifiedMMRProof(
		mmrProof.BlockHash,
		uint64(mmrProof.Proof.LeafIndex),
		mmrProof.Leaf,
		uint64(mmrProof.Proof.LeafCount),
		mmrProof.Proof.Items,
	)
	if err != nil {
		return nil, fmt.Errorf("simplify MMR leaf proof: %w", err)
	}

	mmrRootHash, err := li.relaychainConn.GetMMRRootHash(latestBeefyBlockHash)
	if err != nil {
		return nil, fmt.Errorf("retrieve MMR root hash at block %v: %w", latestBeefyBlockHash.Hex(), err)
	}

	var merkleProofData *MerkleProofData
	merkleProofData, input.ParaHeads, err = li.generateAndValidateParasHeadsMerkleProof(input, &mmrProof)
	if err != nil {
		return nil, err
	}

	log.Debug("Created all parachain merkle proof data")

	output := ProofOutput{
		MMRProof:        simplifiedProof,
		MMRRootHash:     mmrRootHash,
		Header:          *header,
		MerkleProofData: *merkleProofData,
	}

	return &output, nil
}

// Generate a merkle proof for the parachain head with input ParaId and verify with merkle root hash of all parachain heads
func (li *BeefyListener) generateAndValidateParasHeadsMerkleProof(input *ProofInput, mmrProof *types.GenerateMMRProofResponse) (*MerkleProofData, []relaychain.ParaHead, error) {
	// Polkadot uses the following code to generate merkle root from parachain headers:
	// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/westend/src/lib.rs#L453-L460
	// Truncate the ParaHeads to the 1024
	// https://github.com/paritytech/polkadot-sdk/blob/d66dee3c3da836bcf41a12ca4e1191faee0b6a5b/polkadot/runtime/parachains/src/paras/mod.rs#L1305-L1311
	paraHeads := input.ParaHeads
	numParas := min(MaxParaHeads, len(paraHeads))
	merkleProofData, err := CreateParachainMerkleProof(paraHeads[:numParas], input.ParaID)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("create parachain header proof: %w", err)
	}

	// Verify merkle root generated is same as value generated in relaychain and if so exit early
	if merkleProofData.Root.Hex() == mmrProof.Leaf.ParachainHeads.Hex() {
		return &merkleProofData, paraHeads, nil
	}

	// Try a filtering out parathreads
	log.WithFields(log.Fields{
		"computedMmr": merkleProofData.Root.Hex(),
		"mmr":         mmrProof.Leaf.ParachainHeads.Hex(),
	}).Warn("MMR parachain merkle root does not match calculated merkle root. Trying to filtering out parathreads.")

	paraHeads, err = li.relaychainConn.FilterParachainHeads(paraHeads, input.RelayBlockHash)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("could not filter out parathreads: %w", err)
	}

	numParas = min(MaxParaHeads, len(paraHeads))
	merkleProofData, err = CreateParachainMerkleProof(paraHeads[:numParas], input.ParaID)
	if err != nil {
		return nil, paraHeads, fmt.Errorf("create parachain header proof: %w", err)
	}
	if merkleProofData.Root.Hex() != mmrProof.Leaf.ParachainHeads.Hex() {
		return nil, paraHeads, fmt.Errorf("MMR parachain merkle root does not match calculated parachain merkle root (mmr: %s, computed: %s)",
			mmrProof.Leaf.ParachainHeads.Hex(),
			merkleProofData.Root.String(),
		)
	}
	return &merkleProofData, paraHeads, nil
}

func (li *BeefyListener) waitAndSend(ctx context.Context, task *Task, waitingPeriod uint64) error {
	paraNonce := (*task.MessageProofs)[0].Message.Nonce
	log.Info(fmt.Sprintf("waiting for nonce %d to be picked up by another relayer", paraNonce))
	var cnt uint64
	var err error
	for {
		isRelayed, err := li.scanner.isNonceRelayed(ctx, uint64(paraNonce))
		if err != nil {
			return err
		}
		if isRelayed {
			log.Info(fmt.Sprintf("nonce %d picked up by another relayer, just skip", paraNonce))
			return nil
		}
		if cnt == waitingPeriod {
			break
		}
		time.Sleep(time.Duration(li.scheduleConfig.SleepInterval) * time.Second)
		cnt++
	}
	log.Info(fmt.Sprintf("nonce %d is not picked up by any one, submit anyway", paraNonce))
	task.ProofOutput, err = li.generateProof(ctx, task.ProofInput, task.Header)
	if err != nil {
		return err
	}
	select {
	case <-ctx.Done():
		return ctx.Err()
	case li.tasks <- task:
		log.Info("Beefy Listener emitted new task")
	}
	return nil
}
