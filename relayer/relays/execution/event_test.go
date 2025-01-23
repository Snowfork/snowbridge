package execution

import (
	"context"
	"fmt"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/snowfork/snowbridge/relayer/contracts"
	assert "github.com/stretchr/testify/require"
	"testing"
)

func TestHexToBytes(t *testing.T) {
	config := SourceConfig{
		Ethereum: config.EthereumConfig{
			Endpoint: "ws://127.0.0.1:8546/",
		},
		Contracts: ContractsConfig{
			Gateway: "0xb8ea8cb425d85536b158d661da1ef0895bb92f1d",
		},
	}
	ethconn := ethereum.NewConnection(&config.Ethereum, nil)

	err := ethconn.Connect(context.Background())
	assert.NoError(t, err)

	address := common.HexToAddress(config.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, ethconn.Client())
	assert.NoError(t, err)

	start := uint64(0)
	end := uint64(1000)
	opts := bind.FilterOpts{
		Start:   start,
		End:     &end,
		Context: context.Background(),
	}
	iter, err := contract.FilterOutboundMessageAccepted(&opts)
	assert.NoError(t, err)

	var events []*contracts.GatewayOutboundMessageAccepted
	done := false

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			assert.NoError(t, err)
			break
		}
		if iter.Event.Nonce >= start {
			events = append(events, iter.Event)
		}

		if iter.Event.Nonce == start && opts.Start != 0 {
			// This iteration of findEventsWithFilter contains the last nonce we are interested in,
			// although the nonces might not be ordered in ascending order in the iterator. So there might be more
			// nonces that need to be appended (and we need to keep looping until "more" is false, even though we
			// already have found the oldest nonce.
			done = true
		}
	}

	if done {
		iter.Close()
	}

	fmt.Printf("events: %v", events)
}
