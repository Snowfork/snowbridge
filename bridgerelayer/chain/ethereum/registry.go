package ethereum

import (
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	log "github.com/sirupsen/logrus"
)

// LoadApplications loads each registered application's ID and application binary interface (ABI)
func LoadApplications(registryPath string) (apps []Application) {
	files, err := ioutil.ReadDir(registryPath)
	if err != nil {
		log.Fatal(err)
	}

	for _, file := range files {
		app := loadApplication(registryPath, file.Name())
		apps = append(apps, app)
	}
	return apps
}

func loadApplication(registryPath string, baseName string) Application {
	jsonFile, err := os.Open(path.Join(registryPath, baseName))
	if err != nil {
		fmt.Println(err)
	}

	defer jsonFile.Close()

	rawABI, _ := ioutil.ReadAll(jsonFile)
	contractABI, err := abi.JSON(strings.NewReader(fmt.Sprintf(`%s`, string(rawABI))))
	if err != nil {
		panic(err)
	}

	return NewApplication(baseName[0:len(baseName)-5], contractABI)
}
