package beefy

import (
	"context"
	"errors"
	"io/ioutil"
	"os"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/jinzhu/gorm"
	_ "github.com/mattn/go-sqlite3" // required by gorm
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

func (TaskRecord) TableName() string {
	return "beefy_relay_info"
}

type CmdType int

const (
	Create CmdType = iota // 0
	Update CmdType = iota // 1
	Delete CmdType = iota // 2
)

type DatabaseCmd struct {
	Task         *Task
	Type         CmdType
	Instructions map[string]interface{}
}

func NewDatabaseCmd(task *Task, cmdType CmdType, instructions map[string]interface{}) DatabaseCmd {
	return DatabaseCmd{
		Task:         task,
		Type:         cmdType,
		Instructions: instructions,
	}
}

type Database struct {
	Path     string
	DB       *gorm.DB
	messages <-chan DatabaseCmd
}

func NewDatabase(messages <-chan DatabaseCmd) *Database {
	return &Database{
		Path:     "",
		DB:       nil,
		messages: messages,
	}
}

func (d *Database) Initialize() error {
	tmpfile, err := ioutil.TempFile("", "beefy.*.db")
	if err != nil {
		return nil
	}
	tmpfile.Close()

	db, err := gorm.Open("sqlite3", tmpfile.Name())
	if err != nil {
		return err
	}

	var beefyRelayInfo TaskRecord
	if !db.HasTable(&beefyRelayInfo) {
		db.CreateTable(&beefyRelayInfo)
		db.Model(&beefyRelayInfo)
	}

	d.Path = tmpfile.Name()
	d.DB = db

	return nil
}

func (d *Database) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		var err1, err2 error

		err1 = d.writeLoop(ctx)
		log.WithField("reason", err1).Info("Shutting down beefy DB")

		sqlDB := d.DB.DB()
		if sqlDB != nil {
			err2 = sqlDB.Close()
			if err2 != nil {
				log.WithError(err2).Error("Unable to close DB connection")
			}

			err2 = os.Remove(d.Path)
			if err2 != nil {
				log.WithError(err2).Error("Unable to delete DB file")
			}
		}

		if err1 != nil {
			if errors.Is(err1, context.Canceled) {
				return nil
			}
			return err1
		}

		return nil
	})

	return nil
}

func (d *Database) writeLoop(ctx context.Context) error {
	var mutex = &sync.Mutex{}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case cmd := <-d.messages:
			mutex.Lock()
			switch cmd.Type {
			case Create:
				tx := d.DB.Begin()
				if err := tx.Error; err != nil {
					return err
				}

				record, err := cmd.Task.Freeze()
				if err != nil {
					return err
				}

				if err := tx.Create(record).Error; err != nil {
					tx.Rollback()
					return err
				}

				if err := tx.Commit().Error; err != nil {
					return err
				}
			case Update:
				if err := d.DB.Model(&cmd.Task).Updates(cmd.Instructions).Error; err != nil {
					return err
				}
			case Delete:
				if err := d.DB.Delete(&cmd.Task).Error; err != nil {
					return err
				}
			}
			mutex.Unlock()
		}
	}
}

func (d *Database) GetTasksByStatus(status Status) ([]*Task, error) {
	records := make([]*TaskRecord, 0)
	err := d.DB.Where("status = ?", status).Find(&records).Error
	if err != nil {
		return nil, err
	}

	var tasks []*Task
	for _, record := range records {
		task, err := record.Thaw()
		if err != nil {
			return nil, err
		}
		tasks = append(tasks, task)
	}

	return tasks, nil
}

func (d *Database) GetTaskByValidationID(id int64) (*Task, error) {
	var record TaskRecord
	err := d.DB.Take(&record, "validation_id = ?", id).Error
	if err != nil {
		return nil, err
	}

	task, err := record.Thaw()
	if err != nil {
		return nil, err
	}

	return task, nil
}

func (d *Database) GetTaskByInitialVerificationTx(txHash common.Hash) (*Task, error) {
	var record TaskRecord
	err := d.DB.Take(&record, "initial_verification_tx = ?", txHash).Error
	if err != nil {
		return nil, err
	}

	task, err := record.Thaw()
	if err != nil {
		return nil, err
	}

	return task, nil
}

func (d *Database) GetTaskByFinalVerificationTx(txHash common.Hash) (*Task, error) {
	var record TaskRecord
	err := d.DB.Take(&record, "final_verification_tx = ?", txHash).Error
	if err != nil {
		return nil, err
	}

	task, err := record.Thaw()
	if err != nil {
		return nil, err
	}

	return task, nil
}
