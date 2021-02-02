// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"bytes"
	"context"
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	gethTrie "github.com/ethereum/go-ethereum/trie"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
)

func getMessagesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "getmessages",
		Short:   "Retrieve the messages specified by block and index",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay getmessages -b 812e7d414071648252bb3c2dc9c6d2f292fb615634606f9251191c7372eb4acc -i 123",
		RunE:    GetMessagesFn,
	}
	cmd.Flags().StringP("block", "b", "", "Block hash")
	cmd.Flags().UintP(
		"index",
		"i",
		0,
		"Index in the block of the receipt (or transaction) that contains the event",
	)
	cmd.Flags().BoolP(
		"filter-by-config",
		"f",
		false,
		"Select logs based on the config instead of the receipt index",
	)
	cmd.MarkFlagRequired("block")
	return cmd
}

func GetMessagesFn(cmd *cobra.Command, _ []string) error {
	config, err := core.LoadConfig()
	if err != nil {
		return err
	}

	hashBytes, err := hex.DecodeString(cmd.Flags().Lookup("block").Value.String())
	if err != nil {
		return err
	}
	blockHash := gethCommon.BytesToHash(hashBytes)
	index, err := strconv.ParseUint(cmd.Flags().Lookup("index").Value.String(), 10, 16)
	if err != nil {
		return err
	}
	filterByConfig := cmd.Flags().Lookup("filter-by-config").Value.String() == "true"
	indexPtr := &index
	if filterByConfig {
		indexPtr = nil
	}

	logs, trie, err := getEthLogsAndReceiptTrie(&config.Eth, blockHash, indexPtr)
	if err != nil {
		return err
	}
	for _, log := range logs {
		printEthLogForSub(log, trie)
	}
	return nil
}

func getEthLogsAndReceiptTrie(
	config *ethereum.Config,
	blockHash gethCommon.Hash,
	index *uint64,
) ([]*gethTypes.Log, *gethTrie.Trie, error) {
	ctx := context.Background()
	kp, err := secp256k1.NewKeypairFromString(config.PrivateKey)
	if err != nil {
		return nil, nil, err
	}
	log := logrus.WithField("chain", "Ethereum")
	conn := ethereum.NewConnection(config.Endpoint, kp, log)
	err = conn.Connect(ctx)
	if err != nil {
		return nil, nil, err
	}
	defer conn.Close()

	loader := ethereum.DefaultBlockLoader{Conn: conn}
	block, err := loader.GetBlock(ctx, blockHash)
	if err != nil {
		return nil, nil, err
	}

	receipts, err := loader.GetAllReceipts(ctx, block)
	if err != nil {
		return nil, nil, err
	}
	trie := makeTrie(receipts)

	if index == nil {
		contracts, err := ethereum.LoadAppContracts(config)
		if err != nil {
			return nil, nil, err
		}

		query := ethereum.MakeFilterQuery(contracts)
		query.BlockHash = &blockHash
		logs, err := conn.GetClient().FilterLogs(ctx, *query)
		if err != nil {
			return nil, nil, err
		}

		logPtrs := make([]*gethTypes.Log, len(logs))
		for i, log := range logs {
			logCopy := log
			logPtrs[i] = &logCopy
		}
		return logPtrs, trie, nil
	}

	receipt := receipts[*index]
	return receipt.Logs, trie, nil
}

func printEthLogForSub(log *gethTypes.Log, trie *gethTrie.Trie) error {
	message, err := ethereum.MakeMessageFromEvent(log, trie, logrus.WithField("chain", "Ethereum"))
	if err != nil {
		return err
	}
	messageEth := message.Payload.(ethereum.Message)
	verificationInput := messageEth.VerificationInput.AsReceiptProof
	formatProofVec := func(data []types.Bytes) string {
		hexRep := make([]string, len(data))
		for i, datum := range data {
			hexRep[i] = fmt.Sprintf("hex!(\"%s\").to_vec()", hex.EncodeToString(datum))
		}
		return fmt.Sprintf(`vec![
			%s,
		]`, strings.Join(hexRep, ",\n"))
	}
	fmt.Println("")
	fmt.Printf(
		`Message {
			payload: hex!("%s").to_vec(),
			verification: VerificationInput::ReceiptProof {
				block_hash: hex!("%x").into(),
				tx_index: %d,
				proof: (
					%s,
					%s,
				),
			},
		}`,
		hex.EncodeToString(messageEth.Data),
		verificationInput.BlockHash,
		verificationInput.TxIndex,
		formatProofVec(verificationInput.Proof.Keys),
		formatProofVec(verificationInput.Proof.Values),
	)
	fmt.Println("")
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
