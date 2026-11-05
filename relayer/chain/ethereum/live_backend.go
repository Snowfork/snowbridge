// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
)

// LiveClient is a stable bind.ContractBackend returned by Connection.Client().
// abigen BoundContract stores the backend pointer given at NewGateway/NewBeefyClient/
// NewMulticall3; this wrapper is allocated once and always forwards to the
// *ethclient.Client currently held after Reconnect, instead of a websocket that
// Reconnect already closed.
type LiveClient struct {
	conn *Connection
}

var (
	_ bind.ContractBackend       = (*LiveClient)(nil)
	_ bind.PendingContractCaller = (*LiveClient)(nil)
)

func (l *LiveClient) live() (*ethclient.Client, error) {
	if l == nil || l.conn == nil {
		return nil, fmt.Errorf("ethereum client is not connected")
	}
	return l.conn.liveClient()
}

func (co *Connection) liveClient() (*ethclient.Client, error) {
	co.mu.Lock()
	defer co.mu.Unlock()
	if co.client == nil {
		return nil, fmt.Errorf("ethereum client is not connected")
	}
	return co.client, nil
}

func (l *LiveClient) CodeAt(ctx context.Context, account common.Address, blockNumber *big.Int) ([]byte, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.CodeAt(ctx, account, blockNumber)
}

func (l *LiveClient) CallContract(ctx context.Context, msg ethereum.CallMsg, blockNumber *big.Int) ([]byte, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.CallContract(ctx, msg, blockNumber)
}

func (l *LiveClient) PendingCodeAt(ctx context.Context, account common.Address) ([]byte, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.PendingCodeAt(ctx, account)
}

func (l *LiveClient) PendingCallContract(ctx context.Context, msg ethereum.CallMsg) ([]byte, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.PendingCallContract(ctx, msg)
}

func (l *LiveClient) HeaderByNumber(ctx context.Context, number *big.Int) (*types.Header, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.HeaderByNumber(ctx, number)
}

func (l *LiveClient) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.HeaderByHash(ctx, hash)
}

func (l *LiveClient) PendingNonceAt(ctx context.Context, account common.Address) (uint64, error) {
	client, err := l.live()
	if err != nil {
		return 0, err
	}
	return client.PendingNonceAt(ctx, account)
}

func (l *LiveClient) NonceAt(ctx context.Context, account common.Address, blockNumber *big.Int) (uint64, error) {
	client, err := l.live()
	if err != nil {
		return 0, err
	}
	return client.NonceAt(ctx, account, blockNumber)
}

func (l *LiveClient) SuggestGasPrice(ctx context.Context) (*big.Int, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.SuggestGasPrice(ctx)
}

func (l *LiveClient) SuggestGasTipCap(ctx context.Context) (*big.Int, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.SuggestGasTipCap(ctx)
}

func (l *LiveClient) EstimateGas(ctx context.Context, msg ethereum.CallMsg) (uint64, error) {
	client, err := l.live()
	if err != nil {
		return 0, err
	}
	return client.EstimateGas(ctx, msg)
}

func (l *LiveClient) SendTransaction(ctx context.Context, tx *types.Transaction) error {
	// Optional private/builder submit RPC: only eth_sendRawTransaction leaves the
	// read Endpoint (Alchemy/public). All other ContractBackend methods stay put.
	if endpoint := l.conn.SubmitEndpoint(); endpoint != "" {
		return l.conn.SendTransactionViaSubmitEndpoint(ctx, tx)
	}
	client, err := l.live()
	if err != nil {
		return err
	}
	return client.SendTransaction(ctx, tx)
}

func (l *LiveClient) FilterLogs(ctx context.Context, q ethereum.FilterQuery) ([]types.Log, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.FilterLogs(ctx, q)
}

func (l *LiveClient) SubscribeFilterLogs(ctx context.Context, q ethereum.FilterQuery, ch chan<- types.Log) (ethereum.Subscription, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.SubscribeFilterLogs(ctx, q, ch)
}

func (l *LiveClient) SubscribeNewHead(ctx context.Context, ch chan<- *types.Header) (ethereum.Subscription, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.SubscribeNewHead(ctx, ch)
}

func (l *LiveClient) NetworkID(ctx context.Context) (*big.Int, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.NetworkID(ctx)
}

func (l *LiveClient) BlockNumber(ctx context.Context) (uint64, error) {
	client, err := l.live()
	if err != nil {
		return 0, err
	}
	return client.BlockNumber(ctx)
}

func (l *LiveClient) TransactionByHash(ctx context.Context, hash common.Hash) (tx *types.Transaction, isPending bool, err error) {
	client, err := l.live()
	if err != nil {
		return nil, false, err
	}
	return client.TransactionByHash(ctx, hash)
}

func (l *LiveClient) TransactionReceipt(ctx context.Context, txHash common.Hash) (*types.Receipt, error) {
	client, err := l.live()
	if err != nil {
		return nil, err
	}
	return client.TransactionReceipt(ctx, txHash)
}

// CallContext issues a raw JSON-RPC call against the live connection.
//
// The Gloas execution proof needs the block header exactly as the node serves it, including
// fields go-ethereum's Header struct does not know and would drop, so that path reads raw
// JSON rather than a decoded header.
func (l *LiveClient) CallContext(ctx context.Context, result interface{}, method string, args ...interface{}) error {
	client, err := l.live()
	if err != nil {
		return err
	}
	return client.Client().CallContext(ctx, result, method, args...)
}
