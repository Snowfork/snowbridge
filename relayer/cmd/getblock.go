// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
)

type Format string

const (
	RustFmt Format = "rust"
	JSONFmt Format = "json"
)

func getBlockCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "getblock",
		Short:   "Retrieve the latest finalized block",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay getblock",
		RunE:    GetBlockFn,
	}
	cmd.Flags().StringP(
		"format",
		"f",
		"rust",
		"The output format. Options are 'rust' and 'json'. They correspond to the Substrate genesis config formats.",
	)
	return cmd
}

func GetBlockFn(cmd *cobra.Command, _ []string) error {
	config, err := core.LoadConfig()
	if err != nil {
		return err
	}

	format := Format(cmd.Flags().Lookup("format").Value.String())

	header, err := getEthBlock(&config.Eth)
	if err != nil {
		return err
	}

	return printEthBlockForSub(header, format)
}

func getEthBlock(config *ethereum.Config) (*gethTypes.Header, error) {
	ctx := context.Background()
	client, err := ethclient.Dial(config.Endpoint)
	if err != nil {
		return nil, err
	}
	defer client.Close()

	chainID, err := client.NetworkID(ctx)
	logrus.WithFields(logrus.Fields{
		"endpoint": config.Endpoint,
		"chainID":  chainID,
	}).Info("Connected to chain")

	latestHeader, err := client.HeaderByNumber(ctx, nil)
	header, err := client.HeaderByNumber(
		ctx,
		new(big.Int).Sub(latestHeader.Number, big.NewInt(int64(config.DescendantsUntilFinal))),
	)
	if err != nil {
		return nil, err
	}

	return header, nil
}

func printEthBlockForSub(header *gethTypes.Header, format Format) error {
	headerForSub, err := ethereum.MakeHeaderData(header)
	if err != nil {
		return err
	}

	fmt.Println("")
	if format == RustFmt {
		fmt.Printf(
			`EthereumHeader {
			parent_hash: hex!("%x").into(),
			timestamp: %du64.into(),
			number: %du64.into(),
			author: hex!("%x").into(),
			transactions_root: hex!("%x").into(),
			ommers_hash: hex!("%x").into(),
			extra_data: hex!("%x").into(),
			state_root: hex!("%x").into(),
			receipts_root: hex!("%x").into(),
			logs_bloom: (&hex!("%x")).into(),
			gas_used: %du64.into(),
			gas_limit: %du64.into(),
			difficulty: %du64.into(),
			seal: vec![
				hex!("%x").to_vec(),
				hex!("%x").to_vec(),
			],
		}`,
			headerForSub.ParentHash,
			header.Time,
			headerForSub.Number,
			headerForSub.Author,
			headerForSub.TransactionsRoot,
			headerForSub.OmmersHash,
			headerForSub.ExtraData,
			headerForSub.StateRoot,
			headerForSub.ReceiptsRoot,
			headerForSub.LogsBloom,
			headerForSub.GasUsed,
			headerForSub.GasLimit,
			headerForSub.Difficulty,
			headerForSub.Seal[0],
			headerForSub.Seal[1],
		)
		fmt.Println("")
	} else {
		extraData, err := json.Marshal(bytesAsArray64(headerForSub.ExtraData))
		if err != nil {
			return err
		}
		logsBloom, err := json.Marshal(headerForSub.LogsBloom)
		if err != nil {
			return err
		}
		seal1, err := json.Marshal(bytesAsArray64(headerForSub.Seal[0]))
		if err != nil {
			return err
		}
		seal2, err := json.Marshal(bytesAsArray64(headerForSub.Seal[1]))
		if err != nil {
			return err
		}

		fmt.Printf(
			`{
			"parent_hash": "%s",
			"timestamp": %d,
			"number": %d,
			"author": "%s",
			"transactions_root": "%s",
			"ommers_hash": "%s",
			"extra_data": %s,
			"state_root": "%s",
			"receipts_root": "%s",
			"logs_bloom": %s,
			"gas_used": "%#x",
			"gas_limit": "%#x",
			"difficulty": "%#x",
			"seal": [
				%s,
				%s
			]
		}`,
			headerForSub.ParentHash.Hex(),
			header.Time,
			headerForSub.Number,
			headerForSub.Author.Hex(),
			headerForSub.TransactionsRoot.Hex(),
			headerForSub.OmmersHash.Hex(),
			extraData,
			headerForSub.StateRoot.Hex(),
			headerForSub.ReceiptsRoot.Hex(),
			logsBloom,
			headerForSub.GasUsed,
			headerForSub.GasLimit,
			headerForSub.Difficulty,
			seal1,
			seal2,
		)
		fmt.Println("")
	}

	return nil
}

func bytesAsArray64(bytes []byte) []uint64 {
	arr := make([]uint64, len(bytes))
	for i, v := range bytes {
		arr[i] = uint64(v)
	}
	return arr
}
