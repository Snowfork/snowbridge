// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/validatorregistry"
	"github.com/snowfork/polkadot-ethereum/relayer/parachain"
)

type Writer struct {
	config   *Config
	econn    *ethereum.Connection
	conn     *Connection
	messages <-chan []chain.Message
	beefy    chan parachain.BeefyCommitmentInfo
	log      *logrus.Entry
	// TODO: generalize contracts
	lightclientbridge *lightclientbridge.Contract
	valregistry       *validatorregistry.Contract
}

func NewWriter(config *Config, conn *Connection, econn *ethereum.Connection, messages <-chan []chain.Message,
	beefy chan parachain.BeefyCommitmentInfo, log *logrus.Entry) (*Writer, error) {
	return &Writer{
		config:   config,
		conn:     conn,
		econn:    econn,
		messages: messages,
		beefy:    beefy,
		log:      log,
	}, nil
}

func (wr *Writer) Start(ctx context.Context, eg *errgroup.Group) error {
	lightClientBridgeContract, err := lightclientbridge.NewContract(common.HexToAddress(wr.config.Ethereum.Contracts.RelayBridgeLightClient), wr.econn.GetClient())
	if err != nil {
		return err
	}
	wr.lightclientbridge = lightClientBridgeContract

	validatorRegistryContract, err := validatorregistry.NewContract(common.HexToAddress(wr.config.Ethereum.Contracts.ValidatorRegistry), wr.econn.GetClient())
	if err != nil {
		return err
	}
	wr.valregistry = validatorRegistryContract

	eg.Go(func() error {
		return wr.writeLoop(ctx)
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

func (wr *Writer) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case msgs := <-wr.messages:
			for _, msg := range msgs {
				beefyInfo, ok := msg.(parachain.BeefyCommitmentInfo)
				if !ok {
					return fmt.Errorf("Invalid message")
				}

				wr.log.Info("Parachain writer: processing new beefyInfo with status: ", beefyInfo.Status)

				switch beefyInfo.Status {
				case parachain.CommitmentWitnessed:
					err := wr.WriteNewSignatureCommitment(ctx, beefyInfo)
					if err != nil {
						wr.log.WithError(err).Error("Error submitting message to ethereum")
					}
				case parachain.InitialVerificationTxSent, parachain.InitialVerificationTxConfirmed:
					continue // Ethereum listener is responsible for checking tx confirmation
				case parachain.ReadyToComplete:
					err := wr.WriteCompleteSignatureCommitment(ctx, beefyInfo)
					if err != nil {
						wr.log.WithError(err).Error("Error submitting message to ethereum")
					}
				default:
					wr.log.Info("Invalid beefy commitment status")
				}
			}
		}
	}
}

func (wr *Writer) signerFn(_ common.Address, tx *gethTypes.Transaction) (*gethTypes.Transaction, error) {
	signedTx, err := gethTypes.SignTx(tx, gethTypes.HomesteadSigner{}, wr.econn.GetKeyPair().PrivateKey())
	if err != nil {
		return nil, err
	}
	return signedTx, nil
}

func (wr *Writer) WriteNewSignatureCommitment(ctx context.Context, beefyInfo parachain.BeefyCommitmentInfo) error {
	msg, err := beefyInfo.BuildNewSignatureCommitmentMessage()
	if err != nil {
		return err
	}

	inSet, err := wr.CheckValidatorInSet(ctx, msg.ValidatorPublicKey, msg.ValidatorPublicKeyMerkleProof)
	if err != nil {
		return err
	}
	if !inSet {
		return fmt.Errorf("validator address merkle proof failed verification")
	}

	contract := wr.lightclientbridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.econn.GetKeyPair().CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 5000000, // TODO: reasonable gas limit
	}

	tx, err := contract.NewSignatureCommitment(&options, msg.Payload,
		msg.ValidatorClaimsBitfield, msg.ValidatorSignatureCommitment,
		msg.ValidatorPublicKey, msg.ValidatorPublicKeyMerkleProof)
	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("New Signature Commitment transaction submitted")

	beefyInfo.Status = parachain.InitialVerificationTxSent
	beefyInfo.InitialVerificationTxHash = tx.Hash()
	wr.beefy <- beefyInfo

	return nil
}

// WriteCompleteSignatureCommitment sends a CompleteSignatureCommitment tx to the LightClientBridge contract
func (wr *Writer) WriteCompleteSignatureCommitment(ctx context.Context, beefyInfo parachain.BeefyCommitmentInfo) error {
	wr.log.Info("Parachain WriteCompleteSignatureCommitment()")

	msg, err := beefyInfo.BuildCompleteSignatureCommitmentMessage()
	if err != nil {
		return err
	}

	contract := wr.lightclientbridge
	if contract == nil {
		return fmt.Errorf("Unknown contract")
	}

	options := bind.TransactOpts{
		From:     wr.econn.GetKeyPair().CommonAddress(),
		Signer:   wr.signerFn,
		Context:  ctx,
		GasLimit: 500000, // TODO: reasonable gas limit
	}

	tx, err := contract.CompleteSignatureCommitment(&options, msg.ID, msg.Payload, msg.RandomSignatureCommitments,
		msg.RandomSignatureBitfieldPositions, msg.RandomValidatorAddresses, msg.RandomPublicKeyMerkleProofs)

	if err != nil {
		wr.log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("Complete Signature Commitment transaction submitted")

	return nil
}

// CheckValidatorInSet checks if a validator address is in the validator set
func (wr *Writer) CheckValidatorInSet(ctx context.Context, valAddr common.Address, valAddrMerkleProof [][32]byte) (bool, error) {
	wr.log.Info("Parachain CheckValidatorInSet()")

	contract := wr.valregistry
	if contract == nil {
		return false, fmt.Errorf("Unknown contract")
	}

	res, err := contract.CheckValidatorInSet(&bind.CallOpts{}, valAddr, valAddrMerkleProof)
	if err != nil {
		return false, err
	}

	return res, nil
}

type LeafProof struct {
	BlockHash types.Hash
	Leaf      types.Bytes
	Proof     types.Data
}

// GenerateMMRProof generates an MMR proof onchain
func (wr *Writer) GenerateMmrProofOnchain(ctx context.Context, beefyInfo parachain.BeefyCommitmentInfo, valAddrIndex int) (LeafProof, error) {
	leafProof := LeafProof{}

	blockNumber := uint64(beefyInfo.SignedCommitment.Commitment.BlockNumber)
	blockHash, err := wr.conn.GetBlockHash(blockNumber)
	if err != nil {
		return leafProof, err
	}

	// TODO: Approach 1: Use Substrate client.
	// 		 Complete example here: https://github.com/Snowfork/go-substrate-rpc-client/pull/3/files

	err = wr.conn.api.Client.Call(&leafProof, "mmr_generateProof", valAddrIndex, blockHash.Hex())
	if err != nil {
		return leafProof, err
	}
	return leafProof, nil

	// TODO: Approach 2: Execute curl cmd. This cmd works from terminal:
	// $ curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "mmr_generateProof",  "params": [0, "0xb02a5671922f1b68b16e6e89628a41aabf0813adc09100161c3b8a68b9ecddb5"]}' http://localhost:9933/

	// curl := exec.Command("curl", "-H", "Content-Type: application/json", "-d", "'{\"id\":1, \"jsonrpc\":\"2.0\", \"method\": \"mmr_generateProof\",  \"params\": [0, \"0xb02a5671922f1b68b16e6e89628a41aabf0813adc09100161c3b8a68b9ecddb5\"]}'", "http://localhost:9933/")
	// output, err := curl.Output()
	// if err != nil {
	// 	panic(err)
	// }

	// leafProof := LeafProof{}
	// err = json.Unmarshal(output, &leafProof)
	// if err != nil {
	// 	panic(err)
	// }
	// return leafProof, nil

	// TODO: Approach 3: HTTP GET request.

	// var localClient = &http.Client{Timeout: 10 * time.Second}
	// url := "http://localhost:9933/mmr_generateProof"
	// req, err := http.NewRequest(http.MethodGet, url, nil)
	// if err != nil {
	// 	wr.log.Error(err)
	// }

	// req.Header.Set("Content-Type", "application/json")
	// q := req.URL.Query()
	// q.Add("index", "0")
	// q.Add("blockHash", "0xb02a5671922f1b68b16e6e89628a41aabf0813adc09100161c3b8a68b9ecddb5")
	// req.URL.RawQuery = q.Encode()

	// res, getErr := localClient.Do(req)
	// if getErr != nil {
	// 	wr.log.Error(err)
	// }

	// if res.Body != nil {
	// 	defer res.Body.Close()
	// }

	// body, readErr := ioutil.ReadAll(res.Body)
	// if readErr != nil {
	// 	wr.log.Error(err)
	// }

	// proof1 := LeafProof{}
	// jsonErr := json.Unmarshal(body, &proof1)
	// if jsonErr != nil {
	// 	wr.log.Error(err)
	// }

	// return proof1, nil
}
