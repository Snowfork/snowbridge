// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package beefyrelayer

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"os"
	"os/signal"
	"syscall"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/influxdata/influxdb/pkg/testing/assert"
	"github.com/sirupsen/logrus"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
)

func TestWriter(t *testing.T) {
	// ------------------------- Set up Database -------------------------
	dbConfig := store.Config{
		Dialect: "sqlite3",
		DBPath:  "tmp.db",
	}

	db, err := store.PrepareDatabase(&dbConfig)
	if err != nil {
		t.Fatal(err)
		t.Fail()
	}

	dbMessages := make(chan store.DatabaseCmd, 1)
	dbLogger := logrus.WithField("database", "Beefy")
	database := store.NewDatabase(db, dbMessages, dbLogger)

	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)
	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case sig := <-notify:
			logrus.WithField("signal", sig.String()).Info("Received signal")
			cancel()
		}
		return nil
	})

	// Start database update loop
	err = database.Start(ctx, eg)
	if err != nil {
		t.Fatal(err)
	}

	// ------------------------- Set up Ethereum writer -------------------------
	logger, hook := test.NewNullLogger()
	log := logger.WithField("chain", "TestChain")

	// Set up config
	config := ethereum.Config{}
	config.Endpoint = "ws://localhost:8545/"
	config.BeefyPrivateKey = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
	config.BeefyLightClient = "0x8cF6147918A5CBb672703F879f385036f8793a24"

	kpEth, err := secp256k1.NewKeypairFromString(config.BeefyPrivateKey)
	if err != nil {
		t.Fatal(err)
	}

	econn := ethereum.NewConnection(config.Endpoint, kpEth, log)
	err = econn.Connect(ctx)
	if err != nil {
		t.Fatal(err)
	}

	beefyMessages := make(chan store.BeefyRelayInfo, 1)

	writer := NewBeefyEthereumWriter(&config, econn, database, dbMessages, beefyMessages, log)

	err = writer.Start(ctx, eg)
	if err != nil {
		t.Fatal(err)
	}

	// ------------------------- Write Beefy Info to contract -------------------------
	beefyItem := loadSampleBeefyRelayInfo()

	// Send NewSignatureCommitment tx
	err = writer.WriteNewSignatureCommitment(ctx, beefyItem, 0)
	if err != nil {
		t.Fatal(err)
	}

	entries := hook.AllEntries()
	txSendEntry := entries[len(entries)-2]

	assert.Equal(t, txSendEntry.Level, logrus.InfoLevel)
	assert.Equal(t, txSendEntry.Message, "New Signature Commitment transaction submitted")

	txHash := common.HexToHash(txSendEntry.Data["txHash"].(string))

	// Wait for the tx to be confirmed
	isPending := true
	for isPending {
		_, isPending, err = econn.GetClient().TransactionByHash(ctx, txHash)
		if err != nil {
			log.Fatal(err)
		}

		time.Sleep(5 * time.Second)
	}

	// Fetch the tx receipt
	receipt, err := econn.GetClient().TransactionReceipt(ctx, txHash)
	if err != nil {
		log.Fatal(err)
	}
	assert.Equal(t, receipt.TxHash, txHash)
	assert.Equal(t, receipt.Status, uint64(1)) // Successful tx status
}

func loadSampleBeefyRelayInfo() store.BeefyRelayInfo {
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
	beefySig1 := store.BeefySignature(sig1Input)

	sig2 := "daf339e4e248cdc46b4b84640ffc3987ab843ab336b9e39fcf7cc9bb65841b2a0b224d5116c8b0e7b7bb3a99df92d53d4ffe0f7857a753ada3d28be130585ab801"
	sig2Bytes, err := hex.DecodeString(sig2)
	if err != nil {
		panic(err)
	}
	var sig2Input [65]byte
	copy(sig2Input[:], sig2Bytes)
	beefySig2 := store.BeefySignature(sig2Input)

	signedCommitment := store.SignedCommitment{
		Commitment: store.Commitment{
			Payload:        types.NewH256(payloadBytes),
			BlockNumber:    types.NewU32(930),
			ValidatorSetID: types.NewU64(0),
		},
		Signatures: []store.OptionBeefySignature{
			store.NewOptionBeefySignature(beefySig1),
			store.NewOptionBeefySignature(beefySig2),
		},
	}

	valAddrBytes, err := json.Marshal(beefyValidatorAddresses)
	if err != nil {
		panic(err)
	}

	signedCommitmentBytes, err := json.Marshal(signedCommitment)
	if err != nil {
		panic(err)
	}

	return store.BeefyRelayInfo{
		ValidatorAddresses:         valAddrBytes,
		SignedCommitment:           signedCommitmentBytes,
		Status:                     store.CommitmentWitnessed,
		InitialVerificationTxHash:  common.Hash{},
		CompleteOnBlock:            22,
		CompleteVerificationTxHash: common.Hash{},
	}
}
