package beefy

import (
	"context"
	"encoding/hex"
	"os"
	"os/signal"
	"syscall"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/jinzhu/gorm"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/suite"
	"golang.org/x/sync/errgroup"
)

type StoreTestSuite struct {
	suite.Suite

	database *Database
	messages chan DatabaseCmd
	ctx      context.Context
}

func TestStoreTestSuite(t *testing.T) {
	suite.Run(t, new(StoreTestSuite))
}

func (suite *StoreTestSuite) SetupTest() {

	messages := make(chan DatabaseCmd, 1)
	database := NewDatabase(messages)

	err := database.Initialize()
	if err != nil {
		suite.Fail(err.Error())
	}

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

	suite.messages = messages
	suite.database = database
	suite.ctx = ctx

	// Start database update loop
	err = database.Start(ctx, eg)
	if err != nil {
		panic(err)
	}
}

func (suite *StoreTestSuite) TestGetTasksByStatus() {
	task := makeFixture()

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	items, _ := suite.database.GetTasksByStatus(CommitmentWitnessed)
	beefyItem := items[0]
	suite.Equal(beefyItem.Status, CommitmentWitnessed)
}

func (suite *StoreTestSuite) TestGetTaskByValidationID() {
	id := int64(55)
	task := makeFixture()
	task.ValidationID = id

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	foundItem, err := suite.database.GetTaskByValidationID(id)
	if suite.NoError(err) {
		suite.Equal(task.ValidationID, foundItem.ValidationID)
	}
}

func (suite *StoreTestSuite) TestGetTaskByInitialVerificationTx() {
	hash := common.BytesToHash([]byte("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"))
	task := makeFixture()
	task.InitialVerificationTx = hash

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	foundItem, _ := suite.database.GetTaskByInitialVerificationTx(hash)
	suite.Equal(task.InitialVerificationTx, foundItem.InitialVerificationTx)
}

func (suite *StoreTestSuite) TestGetTaskByFinalVerificationTx() {
	hash := common.BytesToHash([]byte("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"))
	task := makeFixture()
	task.FinalVerificationTx = hash

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	foundItem, err := suite.database.GetTaskByFinalVerificationTx(hash)
	if suite.NoError(err) {
		suite.Equal(task.FinalVerificationTx, foundItem.FinalVerificationTx)
	}
}

func (suite *StoreTestSuite) TestUpdateItem() {
	task := makeFixture()

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	items, _ := suite.database.GetTasksByStatus(CommitmentWitnessed)
	beefyItem := items[0]

	// Pass update instruction to write loop
	hash := common.BytesToHash([]byte("0x25451A4de12dcCc2D166922fA938E900fCc4ED24"))
	instructions := map[string]interface{}{
		"status":                  InitialVerificationTxSent,
		"initial_verification_tx": hash,
	}
	updateCmd := NewDatabaseCmd(beefyItem, Update, instructions)
	suite.messages <- updateCmd

	time.Sleep(2 * time.Second)

	newTask, err := suite.database.GetTaskByInitialVerificationTx(hash)
	if suite.NoError(err) {
		suite.Equal(newTask.Status, InitialVerificationTxSent)
		suite.Equal(newTask.InitialVerificationTx, hash)
	}
}

func (suite *StoreTestSuite) TestDeleteItem() {
	id := int64(88)
	task := makeFixture()
	task.ValidationID = id

	// Pass create command to write loop
	createCmd := NewDatabaseCmd(task, Create, nil)
	suite.messages <- createCmd

	time.Sleep(2 * time.Second)

	foundItem, err := suite.database.GetTaskByValidationID(id)
	if suite.NoError(err) {
		suite.Equal(task.ValidationID, foundItem.ValidationID)
		suite.Equal(task.CompleteOnBlock, foundItem.CompleteOnBlock)
	}

	// Pass delete command to write loop
	deleteCmd := NewDatabaseCmd(task, Delete, nil)
	suite.messages <- deleteCmd

	time.Sleep(2 * time.Second)

	_, err = suite.database.GetTaskByValidationID(id)
	suite.ErrorIs(err, gorm.ErrRecordNotFound)
}

func makeFixture() *Task {
	// Sample BEEFY commitment: validator addresses
	validators := []common.Address{
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
	beefySig1 := types.BeefySignature(sig1Input)

	sig2 := "daf339e4e248cdc46b4b84640ffc3987ab843ab336b9e39fcf7cc9bb65841b2a0b224d5116c8b0e7b7bb3a99df92d53d4ffe0f7857a753ada3d28be130585ab801"
	sig2Bytes, err := hex.DecodeString(sig2)
	if err != nil {
		panic(err)
	}
	var sig2Input [65]byte
	copy(sig2Input[:], sig2Bytes)
	beefySig2 := types.BeefySignature(sig2Input)

	signedCommitment := types.SignedCommitment{
		Commitment: types.Commitment{
			Payload: []types.PayloadItem{{
				ID:   [2]byte{109, 104},
				Data: payloadBytes,
			}},
			BlockNumber:    930,
			ValidatorSetID: 0,
		},
		Signatures: []types.OptionBeefySignature{
			types.NewOptionBeefySignature(beefySig1),
			types.NewOptionBeefySignature(beefySig2),
		},
	}

	return &Task{
		TaskRecord: TaskRecord{
			ValidationID:    int64(1),
			Status:          CommitmentWitnessed,
			CompleteOnBlock: 22,
		},
		Validators:       validators,
		SignedCommitment: signedCommitment,
	}
}
