package registry

import (
	"errors"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// DirApps is the directory containing registered applications
const DirApps = "/applications"

var (
	_, b, _, _ = runtime.Caller(0)
	dir        = filepath.Dir(b)
)

// LoadApplications loads each registered application's ID and application binary interface (ABI)
func LoadApplications() (apps []types.Application) {
	files, err := ioutil.ReadDir(dir + DirApps)
	if err != nil {
		log.Fatal(err)
	}

	for _, file := range files {
		app := loadApplication(file.Name())
		apps = append(apps, app)
	}
	return apps
}

func loadApplication(fileName string) types.Application {
	jsonFile, err := os.Open(dir + DirApps + "/" + fileName)
	if err != nil {
		fmt.Println(err)
	}

	defer jsonFile.Close()

	rawABI, _ := ioutil.ReadAll(jsonFile)
	contractABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(rawABI))))
	if err != nil {
		panic(err)
	}

	return types.NewApplication(fileName[0:len(fileName)-5], contractABI)
}

// RegisterApplication registers a new application in the application registry
func RegisterApplication(id common.Address, abi string) error {
	err := validateApplication(id.Hex(), abi)
	if err != nil {
		return err
	}

	err = ioutil.WriteFile(fmt.Sprintf("/applications/%s.json", id.Hex()), []byte(abi), 0644)
	if err != nil {
		return err
	}

	return nil
}

func validateApplication(id, rawABI string) error {
	// Validate that the application's ID is unique
	files, err := ioutil.ReadDir(dir + DirApps)
	if err != nil {
		return err
	}

	for _, file := range files {
		fileName := file.Name()
		if strings.ToUpper(fileName[0:len(fileName)-5]) == strings.ToUpper(id) {
			return fmt.Errorf("application %s is already registered", id)
		}
	}

	// Validate that the raw ABI represents a valid application binary interface
	ABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(rawABI))))
	if err != nil {
		return err
	}

	// Validate that the application implements required named event 'AppEvent'
	appEvent := ABI.Events[types.EventName]
	if appEvent.Name != types.EventName {
		return errors.New("application must implement required named event AppEvent")
	}

	return nil
}

// DeregisterApplication removes an application from the application registry
func DeregisterApplication(ID common.Address) error {
	fileName := ID.Hex() + ".json"
	err := os.Remove(dir + DirApps + "/" + fileName)
	if err != nil {
		return err
	}
	return nil
}
