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
	Hash string `json:"hash"`
	Data string `json:"data"`
}

type BasicChannelEvent struct {
	Hash     types.H256
	Messages []OutboundChannelMessage
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

func (q *QueryClient) QueryEvent(ctx context.Context, api string, blockHash types.Hash) (*BasicChannelEvent, error) {
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
		}).Error("Failed to query event.")
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

	var event *BasicChannelEvent

	for _, item := range items.Items {

		var hash types.H256
		err = types.DecodeFromHexString(item.Hash, &hash)
		if err != nil {
			return nil, err
		}

		var messages []OutboundChannelMessage
		err = types.DecodeFromHexString(item.Data, &messages)
		if err != nil {
			return nil, err
		}
		event = &BasicChannelEvent{
			Hash:     hash,
			Messages: messages,
		}
	}
	log.WithFields(log.Fields{
		"event": event,
	}).Debug("parachain.QueryEvents")

	return event, nil
}
