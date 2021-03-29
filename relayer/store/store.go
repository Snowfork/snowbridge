package store

import (
	"context"
	"encoding/json"
	"io/ioutil"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/jinzhu/gorm"
	_ "github.com/mattn/go-sqlite3"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type BeefyItem struct {
	gorm.Model
	ValidatorAddresses        []byte
	SignedCommitment          []byte
	Status                    Status
	InitialVerificationTxHash common.Hash
	CompleteOnBlock           uint64
}

func NewBeefyItem(validatorAddresses, signedCommitment []byte, status Status,
	initialVerificationTxHash common.Hash, completeOnBlock uint64) BeefyItem {
	return BeefyItem{
		ValidatorAddresses:        validatorAddresses,
		SignedCommitment:          signedCommitment,
		Status:                    status,
		InitialVerificationTxHash: initialVerificationTxHash,
		CompleteOnBlock:           completeOnBlock,
	}
}

func (b *BeefyItem) ToBeefy() (Beefy, error) {
	var validatorAddresses []common.Address
	if err := json.Unmarshal(b.ValidatorAddresses, &validatorAddresses); err != nil {
		return Beefy{}, err
	}

	var signedCommitment SignedCommitment
	if err := json.Unmarshal(b.SignedCommitment, &signedCommitment); err != nil {
		return Beefy{}, err
	}

	return Beefy{
		ValidatorAddresses:        validatorAddresses,
		SignedCommitment:          signedCommitment,
		Status:                    b.Status,
		InitialVerificationTxHash: b.InitialVerificationTxHash,
		CompleteOnBlock:           b.CompleteOnBlock,
	}, nil
}

func (BeefyItem) TableName() string {
	return "beefy_item"
}

type DatabaseCmd struct {
	Item         *BeefyItem
	IsUpdate     bool
	Instructions map[string]interface{}
}

func NewDatabaseCmd(item *BeefyItem, isUpdate bool, instructions map[string]interface{}) DatabaseCmd {
	return DatabaseCmd{
		Item:         item,
		IsUpdate:     isUpdate,
		Instructions: instructions,
	}
}

type Database struct {
	DB       *gorm.DB
	messages <-chan DatabaseCmd
	log      *logrus.Entry
}

func NewDatabase(db *gorm.DB, messages <-chan DatabaseCmd, log *logrus.Entry) *Database {
	return &Database{
		DB:       db,
		messages: messages,
		log:      log,
	}
}

func PrepareDatabase(config *Config) (*gorm.DB, error) {
	config.DBConfig.DBPath = "tmp.db"
	tmpDBFile, err := ioutil.TempFile("", config.DBConfig.DBPath)
	if err != nil {
		return nil, err
	}

	db, err := gorm.Open(config.DBConfig.Dialect, tmpDBFile.Name())
	if err != nil {
		return nil, err
	}

	InitTables(db)

	return db, nil
}

func InitTables(db *gorm.DB) {
	var beefyItem BeefyItem
	if !db.HasTable(&beefyItem) {
		db.CreateTable(&beefyItem)
		db.Model(&beefyItem)
	}
}

func (d *Database) onDone(ctx context.Context) error {
	d.log.Info("Shutting down database...")
	// close(d.messages) TODO: properly close channels
	return ctx.Err()
}

func (d *Database) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return d.writeLoop(ctx)
	})

	return nil
}

func (d *Database) writeLoop(ctx context.Context) error {
	var mutex = &sync.Mutex{}

	for {
		select {
		case <-ctx.Done():
			return d.onDone(ctx)
		case cmd := <-d.messages:
			mutex.Lock()
			if cmd.IsUpdate {
				d.log.Info("Updating item in database...")
				d.DB.Model(&cmd.Item).Updates(cmd.Instructions)
			} else {
				d.log.Info("Creating item in database...")
				tx := d.DB.Begin()
				if err := tx.Error; err != nil {
					d.log.Error(err)
				}

				if err := tx.Create(&cmd.Item).Error; err != nil {
					tx.Rollback()
					d.log.Error(err)
				}

				if err := tx.Commit().Error; err != nil {
					d.log.Error(err)
				}
			}
			mutex.Unlock()
		}
	}
}

func (d *Database) GetItemsByStatus(status Status) []*BeefyItem {
	items := make([]*BeefyItem, 0)
	d.DB.Where("status = ?", status).Find(&items)
	return items
}

func (d *Database) GetItemByInitialVerificationTxHash(txHash common.Hash) *BeefyItem {
	var item BeefyItem
	d.DB.Take(&item, "initial_verification_tx_hash = ?", txHash)
	return &item
}
