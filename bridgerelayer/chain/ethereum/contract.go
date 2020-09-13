// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"os"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/mitchellh/go-homedir"
)

type Contract struct {
	Name    string
	Address common.Address
	ABI     *abi.ABI
}

func LoadContracts(config *Config) ([]Contract, error) {
	contracts := []Contract{}
	for name, app := range config.Apps {
		address := common.HexToAddress(app.Address)

		abiPath, err := homedir.Expand(app.AbiPath)
		if err != nil {
			return nil, err
		}

		abi, err := loadContractABI(abiPath)
		if err != nil {
			return nil, err
		}
		contracts = append(contracts, Contract{Name: name, Address: address, ABI: abi})
	}

	return contracts, nil
}

func loadContractABI(abiPath string) (*abi.ABI, error) {
	f, err := os.Open(abiPath)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	contractABI, err := abi.JSON(f)
	if err != nil {
		return nil, err
	}

	return &contractABI, nil
}
