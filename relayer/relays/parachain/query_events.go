package parachain

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os/exec"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type inputItems struct {
	Items []inputItem `json:"items"`
}

type inputItem struct {
	// TODO: remove this channelID
	ID   uint64 `json:"id"`
	Hash string `json:"hash"`
	Data string `json:"data"`
}

// TODO: replace with BasicChannelEvent
type Events struct {
	Basic *BasicChannelEvent
}

type BasicChannelEvent struct {
	Hash    types.H256
	Bundles []BasicOutboundChannelMessageBundle
}

type QueryClient struct {
	NameArgs func(api string, blockHash string) (string, []string)
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

	var outBuf, errBuf bytes.Buffer
	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	err := cmd.Run()
	if err != nil {
		log.WithFields(log.Fields{
			"name":   name,
			"args":   fmt.Sprintf("%v", args),
			"stdErr": errBuf.String(),
			"stdOut": outBuf.String(),
		}).Error("Failed to query events.")
		return nil, err
	}

	var items inputItems
	err = json.Unmarshal(outBuf.Bytes(), &items)
	if err != nil {
		return nil, err
	}

	log.WithFields(log.Fields{
		"inputItems": items,
	}).Debug("parachain.QueryEvents")

	var events Events

	for _, item := range items.Items {

		var hash types.H256
		err = types.DecodeFromHexString(item.Hash, &hash)
		if err != nil {
			return nil, err
		}

		if item.ID == 0 {
			var bundles []BasicOutboundChannelMessageBundle
			err = types.DecodeFromHexString(item.Data, &bundles)
			if err != nil {
				return nil, err
			}
			events.Basic = &BasicChannelEvent{
				Hash:    hash,
				Bundles: bundles,
			}
		} else {
			return nil, fmt.Errorf("unknown channel")
		}
	}
	log.WithFields(log.Fields{
		"events": events,
	}).Debug("parachain.QueryEvents")

	return &events, nil
}
