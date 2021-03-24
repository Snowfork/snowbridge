// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain_test

import (
	"context"
	"encoding/hex"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/influxdata/influxdb/pkg/testing/assert"
	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
	relaychaintypes "github.com/snowfork/polkadot-ethereum/relayer/relaychain"
)

func TestWriter(t *testing.T) {
	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "Relaychain")

	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	defer cancel()

	// Set up config
	config := relaychain.Config{}
	config.Relaychain.Endpoint = "ws://127.0.0.1:9944"
	config.Relaychain.PrivateKey = "//Alice"
	config.Ethereum.Endpoint = "ws://localhost:8545/"
	config.Ethereum.PrivateKey = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
	config.Ethereum.Contracts.RelayBridgeLightClient = "0xa9DB236F2D8Bd19b357550718cFfce279397E71C"
	config.Ethereum.Contracts.ValidatorRegistry = "0xf4efca5540a4e606A44Ae492443411df1BB68804"
	config.Ethereum.BeefyBlockDelay = 5

	// Generate keypair from secret
	kpPara, err := sr25519.NewKeypairFromSeed(config.Relaychain.PrivateKey, "")
	if err != nil {
		t.Fatal(err)
	}

	conn := relaychain.NewConnection(config.Relaychain.Endpoint, kpPara.AsKeyringPair(), log)
	err = conn.Connect(ctx)
	if err != nil {
		t.Fatal(err)
	}

	kpEth, err := secp256k1.NewKeypairFromString(config.Ethereum.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}

	econn := ethereum.NewConnection(config.Ethereum.Endpoint, kpEth, log)
	err = econn.Connect(ctx)
	if err != nil {
		t.Fatal(err)
	}

	// Channels
	beefy := make(chan relaychaintypes.BeefyCommitmentInfo, 1)
	messages := make(chan []chain.Message, 1)

	writer, err := relaychain.NewWriter(&config, conn, econn, messages, beefy, log)
	if err != nil {
		t.Fatal(err)
	}

	err = writer.Start(ctx, eg)
	if err != nil {
		t.Fatal(err)
	}

	beefyInfo := loadSampleBeefyCommitmentInfo()

	// Send NewSignatureCommitment tx
	err = writer.WriteNewSignatureCommitment(ctx, beefyInfo)
	if err != nil {
		t.Fatal(err)
	}

	// -------------------------------------------------------------------------------------------------
	// TODO: If offchain MMR proof fails validator registry verification then we can try to use onchain

	// // Simulate building the message, which generates the MMR proof offchain
	// msg, err := beefyInfo.BuildNewSignatureCommitmentMessage()
	// if err != nil {
	// 	t.Fatal(err)
	// }

	// // Build MMR proof onchain
	// mmrProof, err := writer.GenerateMmrProofOnchain(0, beefyInfo)
	// if err != nil {
	// 	t.Fatal(err)
	// }

	// inSet, err := writer.CheckValidatorInSet(ctx, msg.ValidatorPublicKey, mmrProof.Proof)
	// if err != nil {
	// 	t.Fatal(err)
	// }
	// if !inSet {
	// 	t.Fatal("validator address merkle proof failed verification")
	// }
	// -------------------------------------------------------------------------------------------------

	assert.Equal(t, logrus.InfoLevel, hook.LastEntry().Level)
	assert.Equal(t, "New Signature Commitment transaction submitted", hook.LastEntry().Message)

	txHash := common.HexToHash(hook.LastEntry().Data["txHash"].(string))

	// Wait for the tx to be confirmed
	isPending := true
	for isPending {
		_, isPending, err = econn.GetClient().TransactionByHash(ctx, txHash)
		if err != nil {
			log.Fatal(err)
		}

		t.Log("calling TransactionByHash()...")
		time.Sleep(5 * time.Second)
	}

	// Fetch the tx receipt
	receipt, err := econn.GetClient().TransactionReceipt(ctx, txHash)
	if err != nil {
		log.Fatal(err)
	}
	t.Log("Tx Hash:", receipt.TxHash)
	t.Log("Status:", receipt.Status)
	t.Log("Cumulative Gas Used:", receipt.CumulativeGasUsed)
	t.Log("Logs:", receipt.Logs)
}

func TestGenerateMmrProofOffchain(t *testing.T) {
	beefyInfo := loadSampleBeefyCommitmentInfo()

	_, err := beefyInfo.GenerateMmrProofOffchain(0)
	assert.NoError(t, err)
}

// loadSampleBeefyCommitmentInfo generates a sample BeefyCommitmentInfo for testing, hardcoded values
// are from test/scripts/helpers/subscribeBeefyJustifications.js.
func loadSampleBeefyCommitmentInfo() relaychaintypes.BeefyCommitmentInfo {
	// Sample BEEFY commitment: validator addresses
	beefyValidatorAddresses := []common.Address{
		common.HexToAddress("0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b"),
		common.HexToAddress("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"),
	}

	payload := "9db1a857a25a997190c920b7f333eef069804639546acc411e7937a13c592fd2"
	payloadBytes, err := hex.DecodeString(payload)
	if err != nil {
		panic(err)
	}

	sig1 := "d0834df8b658963611deecf57b845f906f517c1a9b9467f31e8dc292d1a131d22ccb2201dc6e49043fce104418442f9d20acaa3c5d86d2ce20285de0e64b057a01"
	sig1Bytes, err := hex.DecodeString(sig1)
	if err != nil {
		panic(err)
	}
	var sig1Input [65]byte
	copy(sig1Input[:], sig1Bytes)
	beefySig1 := relaychaintypes.BeefySignature(sig1Input)

	sig2 := "daf339e4e248cdc46b4b84640ffc3987ab843ab336b9e39fcf7cc9bb65841b2a0b224d5116c8b0e7b7bb3a99df92d53d4ffe0f7857a753ada3d28be130585ab801"
	sig2Bytes, err := hex.DecodeString(sig2)
	if err != nil {
		panic(err)
	}
	var sig2Input [65]byte
	copy(sig2Input[:], sig2Bytes)
	beefySig2 := relaychaintypes.BeefySignature(sig2Input)

	signedCommitment := relaychaintypes.SignedCommitment{
		Commitment: relaychaintypes.Commitment{
			Payload:        types.NewH256(payloadBytes),
			BlockNumber:    types.BlockNumber(930),
			ValidatorSetID: types.NewU64(0),
		},
		Signatures: []relaychaintypes.OptionBeefySignature{
			relaychaintypes.NewOptionBeefySignature(beefySig1),
			relaychaintypes.NewOptionBeefySignature(beefySig2),
		},
	}

	return relaychaintypes.NewBeefyCommitmentInfo(beefyValidatorAddresses, &signedCommitment)
}
