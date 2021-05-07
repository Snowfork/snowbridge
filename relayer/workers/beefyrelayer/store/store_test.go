package store_test

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
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
	"golang.org/x/sync/errgroup"
)

func TestStore(t *testing.T) {
	config := store.Config{
		Dialect: "sqlite3",
		DBPath:  "tmp.db",
	}

	db, err := store.PrepareDatabase(&config)
	if err != nil {
		t.Fatal(err)
		t.Fail()
	}

	messages := make(chan store.DatabaseCmd, 1)
	logger := logrus.WithField("database", "Beefy")
	database := store.NewDatabase(db, messages, logger)

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
		panic(err)
	}

	item := loadSampleBeefyRelayInfo()

	// Pass create command to write loop
	createCmd := store.NewDatabaseCmd(&item, store.Create, nil)
	messages <- createCmd

	time.Sleep(2 * time.Second)

	items := database.GetItemsByStatus(store.CommitmentWitnessed)
	beefyItem := items[0]
	assert.Equal(t, beefyItem.Status, store.CommitmentWitnessed)

	// Pass update instruction to write loop
	hash := common.BytesToHash([]byte("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"))
	instructions := map[string]interface{}{
		"status":                    store.InitialVerificationTxSent,
		"InitialVerificationTxHash": hash,
	}
	updateCmd := store.NewDatabaseCmd(beefyItem, store.Update, instructions)
	messages <- updateCmd

	time.Sleep(2 * time.Second)

	newItem := database.GetItemByInitialVerificationTxHash(hash)
	assert.Equal(t, newItem.Status, store.InitialVerificationTxSent)
	assert.Equal(t, newItem.InitialVerificationTxHash, hash)
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
			BlockNumber:    types.BlockNumber(930),
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
		ValidatorAddresses:        valAddrBytes,
		SignedCommitment:          signedCommitmentBytes,
		Status:                    store.CommitmentWitnessed,
		InitialVerificationTxHash: common.Hash{},
		CompleteOnBlock:           22,
	}
}
