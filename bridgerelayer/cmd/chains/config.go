package chains

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/ethereum/go-ethereum/log"
)

const DefaultConfigPath = "./test_config.json"
const DefaultKeystorePath = "./keys"

// Config ...
type Config struct {
	Chains []ChainConfig `toml:"chains" json:"chains"`
	// KeystorePath string        `toml:"keystore_path,omitempty" json:"keystorePath,omitempty"`
}

// ChainConfig is parsed directly from the config file and is used to construct the Chain
type ChainConfig struct {
	Type       string `toml:"type" json:"type"`
	ID         string `toml:"id" json:"id"`                   // Chain ID
	Endpoint   string `toml:"endpoint" json:"endpoint"`       // url for rpc endpoint
	Operator   string `toml:"operator" json:"operator"`       // operator's address
	PrivateKey string `toml:"private_key" json:"private_key"` // operator's private key
}

// NewConfig ...
func NewConfig() *Config {
	return &Config{
		Chains: []ChainConfig{},
	}
}

func (c *Config) ToJSON(file string) *os.File {
	var (
		newFile *os.File
		err     error
	)

	var raw []byte
	if raw, err = json.Marshal(*c); err != nil {
		log.Warn("error marshalling json", "err", err)
		os.Exit(1)
	}

	newFile, err = os.Create(file)
	if err != nil {
		log.Warn("error creating config file", "err", err)
	}
	_, err = newFile.Write(raw)
	if err != nil {
		log.Warn("error writing to config file", "err", err)
	}

	if err := newFile.Close(); err != nil {
		log.Warn("error closing file", "err", err)
	}
	return newFile
}

func (c *Config) validate() error {
	for _, chain := range c.Chains {
		if chain.Type == "" {
			return fmt.Errorf("required field chain.Type empty for chain %s", chain.ID)
		}
		if chain.Endpoint == "" {
			return fmt.Errorf("required field chain.Endpoint empty for chain %s", chain.ID)
		}
		if chain.Operator == "" {
			return fmt.Errorf("required field chain.Operator empty for chain %s", chain.ID)
		}
		if chain.PrivateKey == "" {
			return fmt.Errorf("required field chain.PrivateKey empty for chain %s", chain.ID)
		}
	}
	return nil
}

func GetConfig() (*Config, error) {
	var config Config

	err := loadConfig(DefaultConfigPath, &config)
	if err != nil {
		return nil, err
	}

	// TODO: private key loaded from keypath file

	err = config.validate()
	if err != nil {
		return nil, err
	}

	return &config, nil
}

func loadConfig(file string, config *Config) error {
	ext := filepath.Ext(file)
	if ext != ".json" {
		return fmt.Errorf("config file extention must be .json")
	}

	fp, err := filepath.Abs(file)
	if err != nil {
		return err
	}

	fpClean := filepath.Clean(fp)
	log.Debug("Loading configuration", "path", fpClean)

	f, err := os.Open(fpClean)
	if err != nil {
		return err
	}

	if err = json.NewDecoder(f).Decode(&config); err != nil {
		return err
	}

	return nil
}
