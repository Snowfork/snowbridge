// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"bytes"
	"context"
	"encoding/hex"
	"strconv"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	gethTrie "github.com/ethereum/go-ethereum/trie"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
)

func getMessageCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "getmessage",
		Short:   "Retrieve the message specified by block and index",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay getmessage 0x812e7d414071648252bb3c2dc9c6d2f292fb615634606f9251191c7372eb4acc 123",
		RunE:    GetMessageFn,
	}
	cmd.Flags().StringP("block", "b", "", "Block hash")
	cmd.Flags().UintP(
		"index",
		"i",
		0,
		"Index in the block for the receipt (or transaction) that contains the event",
	)
	return cmd
}

func GetMessageFn(cmd *cobra.Command, _ []string) error {
	config, err := core.LoadConfig()
	if err != nil {
		return err
	}

	// Parse args
	hashBytes, err := hex.DecodeString(cmd.Flags().Lookup("block").Value.String())
	if err != nil {
		return err
	}
	blockHash := gethCommon.BytesToHash(hashBytes)
	index, err := strconv.ParseUint(cmd.Flags().Lookup("index").Value.String(), 10, 8)
	if err != nil {
		return err
	}

	// Connect
	ctx := context.Background()
	kp, err := secp256k1.NewKeypairFromString(config.Eth.PrivateKey)
	if err != nil {
		return err
	}
	log := logrus.WithField("chain", "Ethereum")
	conn := ethereum.NewConnection(config.Eth.Endpoint, kp, log)
	err = conn.Connect(ctx)
	if err != nil {
		return err
	}
	defer conn.Close()

	// Actually get the data we want
	loader := ethereum.DefaultBlockLoader{Conn: conn}
	block, err := loader.GetBlock(ctx, blockHash)
	receipts, err := loader.GetAllReceipts(ctx, block)
	receipt := []*gethTypes.Receipt(receipts)[index]
	trie := makeTrie(receipts)
	message, err := ethereum.MakeMessageFromEvent(receipt.Logs[0], trie, log)
	if err != nil {
		return err
	}

	return printMessageForSub(message)
}

func printMessageForSub(message *chain.Message) error {
	return nil
}

func makeTrie(items gethTypes.DerivableList) *gethTrie.Trie {
	keybuf := new(bytes.Buffer)
	trie := new(gethTrie.Trie)
	for i := 0; i < items.Len(); i++ {
		keybuf.Reset()
		rlp.Encode(keybuf, uint(i))
		trie.Update(keybuf.Bytes(), items.GetRlp(i))
	}
	return trie
}
