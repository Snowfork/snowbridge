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

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rlp"
	gethTrie "github.com/ethereum/go-ethereum/trie"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/outbound"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

func fetchMessagesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "fetch-messages",
		Short:   "Retrieve the messages specified by block and index",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay getmessages -b 812e7d414071648252bb3c2dc9c6d2f292fb615634606f9251191c7372eb4acc -i 123",
		RunE:    FetchMessagesFn,
	}
	cmd.Flags().StringP("block", "b", "", "Block hash")
	cmd.Flags().UintP(
		"index",
		"i",
		0,
		"Index in the block of the receipt (or transaction) that contains the event",
	)
	cmd.MarkFlagRequired("block")
	return cmd
}

func FetchMessagesFn(cmd *cobra.Command, _ []string) error {
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

	contractEvents, trie, err := getEthContractEventsAndTrie(&config.Eth, blockHash, index)
	if err != nil {
		return err
	}

	for _, event := range contractEvents {
		printEthContractEventForSub(event, trie)
	}
	return nil
}

func getEthContractEventsAndTrie(
	config *ethereum.Config,
	blockHash gethCommon.Hash,
	index uint64,
) ([]*outbound.ContractMessage, *gethTrie.Trie, error) {
	ctx := context.Background()
	kp, err := secp256k1.NewKeypairFromString(config.PrivateKey)
	if err != nil {
		return nil, nil, err
	}

	conn := ethereum.NewConnection(config.Endpoint, kp, logrus.WithField("chain", "Ethereum"))
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

	contracts, err := getEthContractsFromConfig(config, conn.GetClient())
	if err != nil {
		return nil, nil, err
	}

	contractMessages, err := getEthContractMessages(ctx, contracts, block.NumberU64(), index)
	if err != nil {
		return nil, nil, err
	}

	return contractMessages, trie, nil
}

func getEthContractsFromConfig(config *ethereum.Config, client *ethclient.Client) ([]*outbound.Contract, error) {
	contractBasic, err := outbound.NewContract(gethCommon.HexToAddress(config.Channels.Basic.Outbound), client)
	if err != nil {
		return nil, err
	}

	contractIncentivized, err := outbound.NewContract(gethCommon.HexToAddress(config.Channels.Incentivized.Outbound), client)
	if err != nil {
		return nil, err
	}

	return []*outbound.Contract{contractBasic, contractIncentivized}, nil
}

func getEthContractMessages(
	ctx context.Context,
	contracts []*outbound.Contract,
	blockNumber uint64,
	index uint64,
) ([]*outbound.ContractMessage, error) {
	events := make([]*outbound.ContractMessage, 0)
	filterOps := bind.FilterOpts{Start: blockNumber, End: &blockNumber, Context: ctx}
	for _, contract := range contracts {
		iter, err := contract.FilterMessage(&filterOps)
		if err != nil {
			return nil, err
		}

		for {
			more := iter.Next()
			if !more {
				err = iter.Error()
				if err != nil {
					return nil, err
				}
				break
			}

			if uint64(iter.Event.Raw.TxIndex) != index {
				continue
			}
			events = append(events, iter.Event)
		}
	}

	return events, nil
}

func printEthContractEventForSub(contractMsg *outbound.ContractMessage, trie *gethTrie.Trie) error {
	message, err := ethereum.MakeMessageFromEvent(contractMsg, trie, logrus.WithField("chain", "Ethereum"))
	if err != nil {
		return err
	}

	messageForSub := substrate.Message(*message)
	proof := messageForSub.Proof

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
			data: hex!("%s").to_vec(),
			proof: Proof {
				block_hash: hex!("%x").into(),
				tx_index: %d,
				data: (
					%s,
					%s,
				),
			},
		}`,
		hex.EncodeToString(messageForSub.Data),
		proof.BlockHash,
		proof.TxIndex,
		formatProofVec(proof.Data.Keys),
		formatProofVec(proof.Data.Values),
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
