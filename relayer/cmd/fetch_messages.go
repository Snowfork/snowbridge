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
	"github.com/ethereum/go-ethereum/common"
	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	gethTrie "github.com/ethereum/go-ethereum/trie"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/basic"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/incentivized"
	"github.com/snowfork/polkadot-ethereum/relayer/core"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

func fetchMessagesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "fetch-messages",
		Short:   "Retrieve the messages specified by block and index",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay fetch-messages -b 812e7d414071648252bb3c2dc9c6d2f292fb615634606f9251191c7372eb4acc -i 123",
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

	mapping := make(map[common.Address]string)

	contractEvents, trie, err := getEthContractEventsAndTrie(&config.Eth, mapping, blockHash, index)
	if err != nil {
		return err
	}

	for _, event := range contractEvents {
		printEthContractEventForSub(mapping, event, trie)
	}
	return nil
}

func getEthContractEventsAndTrie(
	config *ethereum.Config,
	mapping map[common.Address]string,
	blockHash gethCommon.Hash,
	index uint64,
) ([]*gethTypes.Log, *gethTrie.Trie, error) {
	ctx := context.Background()

	conn := ethereum.NewConnection(config.Endpoint, nil, logrus.WithField("chain", "Ethereum"))
	err := conn.Connect(ctx)
	if err != nil {
		return nil, nil, err
	}
	defer conn.Close()

	basicOutboundChannel, err := basic.NewBasicOutboundChannel(common.HexToAddress(config.Channels.Basic.Outbound), conn.GetClient())
	if err != nil {
		return nil, nil, err
	}

	incentivizedOutboundChannel, err := incentivized.NewIncentivizedOutboundChannel(common.HexToAddress(config.Channels.Incentivized.Outbound), conn.GetClient())
	if err != nil {
		return nil, nil, err
	}

	mapping[common.HexToAddress(config.Channels.Basic.Outbound)] = "BasicInboundChannel.submit"
	mapping[common.HexToAddress(config.Channels.Incentivized.Outbound)] = "IncentivizedInboundChannel.submit"

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

	allEvents := make([]*gethTypes.Log, 0)

	basicEvents, err := getEthBasicMessages(ctx, basicOutboundChannel, block.NumberU64(), index)
	if err != nil {
		return nil, nil, err
	}
	allEvents = append(allEvents, basicEvents...)

	incentivizedEvents, err := getEthIncentivizedMessages(ctx, incentivizedOutboundChannel, block.NumberU64(), index)
	if err != nil {
		return nil, nil, err
	}
	allEvents = append(allEvents, incentivizedEvents...)

	return allEvents, trie, nil
}

func getEthBasicMessages(
	ctx context.Context,
	contract *basic.BasicOutboundChannel,
	blockNumber uint64,
	index uint64,
) ([]*gethTypes.Log, error) {
	events := make([]*gethTypes.Log, 0)
	filterOps := bind.FilterOpts{Start: blockNumber, End: &blockNumber, Context: ctx}

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
		events = append(events, &iter.Event.Raw)
	}

	return events, nil
}

func getEthIncentivizedMessages(
	ctx context.Context,
	contract *incentivized.IncentivizedOutboundChannel,
	blockNumber uint64,
	index uint64,
) ([]*gethTypes.Log, error) {
	events := make([]*gethTypes.Log, 0)
	filterOps := bind.FilterOpts{Start: blockNumber, End: &blockNumber, Context: ctx}

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
		events = append(events, &iter.Event.Raw)
	}

	return events, nil
}

func printEthContractEventForSub(mapping map[common.Address]string, event *gethTypes.Log, trie *gethTrie.Trie) error {
	message, err := ethereum.MakeMessageFromEvent(mapping, event, trie, logrus.WithField("chain", "Ethereum"))
	if err != nil {
		return err
	}

	msgInner, ok := message.Args[0].(substrate.Message)
	if !ok {
		return err
	}

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
		hex.EncodeToString(msgInner.Data),
		msgInner.Proof.BlockHash,
		msgInner.Proof.TxIndex,
		formatProofVec(msgInner.Proof.Data.Keys),
		formatProofVec(msgInner.Proof.Data.Values),
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
