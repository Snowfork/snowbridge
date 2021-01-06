// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package syncer

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum"
	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
)

type HeaderLoader interface {
	HeaderByHash(ctx context.Context, hash gethCommon.Hash) (*gethTypes.Header, error)
	HeaderByNumber(ctx context.Context, number *big.Int) (*gethTypes.Header, error)
	SubscribeNewHead(ctx context.Context, ch chan<- *gethTypes.Header) (ethereum.Subscription, error)
}

type DefaultHeaderLoader struct {
	client *ethclient.Client
}

func NewHeaderLoader(client *ethclient.Client) *DefaultHeaderLoader {
	return &DefaultHeaderLoader{client: client}
}

func (d *DefaultHeaderLoader) HeaderByHash(ctx context.Context, hash gethCommon.Hash) (*gethTypes.Header, error) {
	return d.client.HeaderByHash(ctx, hash)
}

func (d *DefaultHeaderLoader) HeaderByNumber(ctx context.Context, number *big.Int) (*gethTypes.Header, error) {
	return d.client.HeaderByNumber(ctx, number)
}

func (d *DefaultHeaderLoader) SubscribeNewHead(ctx context.Context, ch chan<- *gethTypes.Header) (ethereum.Subscription, error) {
	return d.client.SubscribeNewHead(ctx, ch)
}
