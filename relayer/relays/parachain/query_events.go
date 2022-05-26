package parachain

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os/exec"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type inputItems struct {
	Items []inputItem `json:"items"`
}

type inputItem struct {
	ID uint64 `json:"id"`
	Hash string `json:"hash"`
	Data string `json:"data"`
}

type Events struct {
	Basic *BasicChannelEvent
	Incentivized *IncentivizedChannelEvent
}

type BasicChannelEvent struct {
	Hash types.H256
	Bundle BasicOutboundChannelMessageBundle
}

type IncentivizedChannelEvent struct {
	Hash types.H256
	Bundle IncentivizedOutboundChannelMessageBundle
}

type QueryClient struct {
	NameArgs func (api string, blockHash string) (string, []string)
}

func NewQueryClient() QueryClient {
	return QueryClient{
		NameArgs: func(api string, blockHash string) (string, []string) {
			return "snowbridge-query-events", []string{"--api", api, "--block", blockHash}
		},
	}
}

func (q *QueryClient) QueryEvents(ctx context.Context, api string, blockHash types.Hash) (*Events, error) {
	name, args := q.NameArgs(api, blockHash.Hex())
	cmd := exec.CommandContext(ctx, name, args...)
	var out bytes.Buffer
	cmd.Stdout = &out
	err := cmd.Run()
	if err != nil {
		return nil, err
	}

	var items inputItems
	err = json.Unmarshal(out.Bytes(), &items)
	if err != nil {
		return nil, err
	}

	var events Events

	for _, item := range items.Items {

		var hash types.H256
		err = types.DecodeFromHexString(item.Hash, &hash)
		if err != nil {
			return nil, err
		}

		if item.ID == 0 {
			var bundle BasicOutboundChannelMessageBundle
			err = types.DecodeFromHexString(item.Data, &bundle)
			if err != nil {
				return nil, err
			}
			events.Basic = &BasicChannelEvent {
				Hash: hash,
				Bundle: bundle,
			}
		} else if item.ID == 1 {
			var bundle IncentivizedOutboundChannelMessageBundle
			err = types.DecodeFromHexString(item.Data, &bundle)
			if err != nil {
				return nil, err
			}
			events.Incentivized = &IncentivizedChannelEvent {
				Hash: hash,
				Bundle: bundle,
			}
		} else {
			return nil, fmt.Errorf("unknown channel")
		}
	}

	return &events, nil
}
