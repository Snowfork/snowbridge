// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"math/rand"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum"
	goEthereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"golang.org/x/sync/errgroup"
)

type Connection struct {
	endpoints  []string
	currentIdx int
	mu         sync.Mutex
	kp         *secp256k1.Keypair
	client     *ethclient.Client
	live       *LiveClient
	chainID    *big.Int
	config     *config.EthereumConfig
}

type JsonError interface {
	Error() string
	ErrorCode() int
	ErrorData() interface{}
}

func NewConnection(config *config.EthereumConfig, kp *secp256k1.Keypair) *Connection {
	endpoints := config.ParseEndpoints()
	co := &Connection{
		endpoints:  endpoints,
		currentIdx: rand.Intn(len(endpoints)),
		kp:         kp,
		config:     config,
	}
	co.live = &LiveClient{conn: co}
	return co
}

func (co *Connection) endpoint() string {
	co.mu.Lock()
	defer co.mu.Unlock()
	return co.endpoints[co.currentIdx]
}

// dial connects to endpoint, attaching any configured per-request headers (e.g. an
// X-API-Key for a dry-run/forked-mainnet RPC). With no headers it is a plain ethclient.Dial.
func (co *Connection) dial(ctx context.Context, endpoint string) (*ethclient.Client, error) {
	if co.config == nil || len(co.config.Headers) == 0 {
		return ethclient.Dial(endpoint)
	}
	headers := http.Header{}
	for k, v := range co.config.Headers {
		headers.Set(k, v)
	}
	rpcClient, err := rpc.DialOptions(ctx, endpoint, rpc.WithHeaders(headers))
	if err != nil {
		return nil, err
	}
	return ethclient.NewClient(rpcClient), nil
}

func (co *Connection) connectToEndpoint(ctx context.Context, endpoint string) (*ethclient.Client, *big.Int, error) {
	client, err := co.dial(ctx, endpoint)
	if err != nil {
		return nil, nil, err
	}
	chainID, err := client.NetworkID(ctx)
	if err != nil {
		client.Close()
		return nil, nil, err
	}
	return client, chainID, nil
}

func (co *Connection) ConnectWithHeartBeat(ctx context.Context, eg *errgroup.Group, heartBeat time.Duration) error {
	var lastErr error
	for _, idx := range rand.Perm(len(co.endpoints)) {
		endpoint := co.endpoints[idx]

		client, chainID, err := co.connectToEndpoint(ctx, endpoint)
		if err != nil {
			log.WithFields(logrus.Fields{
				"endpoint": endpoint,
				"error":    err,
			}).Warn("Failed to connect to Ethereum endpoint")
			lastErr = err
			continue
		}

		log.WithFields(logrus.Fields{
			"endpoint": endpoint,
			"chainID":  chainID,
		}).Info("Connected to chain")

		co.mu.Lock()
		co.client = client
		co.chainID = chainID
		co.currentIdx = idx
		co.mu.Unlock()
		lastErr = nil
		break
	}
	if lastErr != nil {
		return fmt.Errorf("failed to connect to any Ethereum endpoint: %w", lastErr)
	}

	if heartBeat.Abs() > 0 {
		ticker := time.NewTicker(heartBeat)

		eg.Go(func() error {
			defer ticker.Stop()
			for {
				select {
				case <-ctx.Done():
					return ctx.Err()
				case <-ticker.C:
					client, err := co.liveClient()
					if err != nil {
						log.WithError(err).Error("Connection heartbeat failed")
						if recErr := co.Reconnect(ctx); recErr != nil {
							log.WithError(recErr).Error("Ethereum reconnect after heartbeat failure failed")
						}
						continue
					}
					_, err = client.NetworkID(ctx)
					if err != nil {
						log.WithField("endpoint", co.endpoint()).Error("Connection heartbeat failed")
						if recErr := co.Reconnect(ctx); recErr != nil {
							log.WithError(recErr).Error("Ethereum reconnect after heartbeat failure failed")
						}
					}
				}
			}
		})
	}

	return nil
}

func (co *Connection) Reconnect(ctx context.Context) error {
	if len(co.endpoints) <= 1 {
		return nil
	}
	co.mu.Lock()
	startIdx := co.currentIdx
	co.mu.Unlock()

	for i := 1; i < len(co.endpoints); i++ {
		idx := (startIdx + i) % len(co.endpoints)
		endpoint := co.endpoints[idx]

		client, chainID, err := co.connectToEndpoint(ctx, endpoint)
		if err != nil {
			log.WithFields(logrus.Fields{
				"endpoint": endpoint,
				"error":    err,
			}).Warn("Failed to reconnect to Ethereum endpoint")
			continue
		}

		co.mu.Lock()
		log.WithFields(logrus.Fields{
			"from": co.endpoints[co.currentIdx],
			"to":   endpoint,
		}).Info("Switched Ethereum endpoint")

		if co.client != nil {
			co.client.Close()
		}
		co.client = client
		co.chainID = chainID
		co.currentIdx = idx
		co.mu.Unlock()
		return nil
	}
	return fmt.Errorf("failed to reconnect to any Ethereum endpoint")
}

func (co *Connection) Close() {
	co.mu.Lock()
	defer co.mu.Unlock()
	if co.client != nil {
		co.client.Close()
	}
}

// Client returns a stable backend that always forwards to the currently
// connected *ethclient.Client. Call sites keep NewGateway(addr, conn.Client());
// BoundContract caches this pointer, so it must not be the inner ethclient that
// Reconnect closes. Returns nil before Connect (existing Client() == nil checks).
func (co *Connection) Client() *LiveClient {
	if _, err := co.liveClient(); err != nil {
		return nil
	}
	return co.live
}

func (co *Connection) Keypair() *secp256k1.Keypair {
	return co.kp
}

func (co *Connection) ChainID() *big.Int {
	return co.chainID
}

// SubmitEndpoint returns the optional submit-only RPC (trimmed), or "" when unset.
func (co *Connection) SubmitEndpoint() string {
	if co == nil || co.config == nil {
		return ""
	}
	return strings.TrimSpace(co.config.SubmitEndpoint)
}

// SendTransactionViaSubmitEndpoint dials SubmitEndpoint with a plain HTTP/WS client
// (no Endpoint headers) and sends the already-signed tx. Used so reads stay on
// Endpoint while broadcasts go through a private pool (e.g. Flashbots Protect).
func (co *Connection) SendTransactionViaSubmitEndpoint(ctx context.Context, tx *types.Transaction) error {
	endpoint := co.SubmitEndpoint()
	if endpoint == "" {
		return fmt.Errorf("submit-endpoint is not configured")
	}
	client, err := ethclient.DialContext(ctx, endpoint)
	if err != nil {
		return fmt.Errorf("dial submit-endpoint: %w", err)
	}
	defer client.Close()

	if err := client.SendTransaction(ctx, tx); err != nil {
		if isBenignSubmitError(err) {
			log.WithFields(logrus.Fields{
				"txHash":   tx.Hash().Hex(),
				"endpoint": endpoint,
				"error":    err.Error(),
			}).Info("Submit-endpoint broadcast already known")
			return nil
		}
		return err
	}
	log.WithFields(logrus.Fields{
		"txHash":   tx.Hash().Hex(),
		"endpoint": endpoint,
		"path":     "submit-endpoint",
	}).Info("Broadcast signed tx via submit-endpoint")
	return nil
}

func isBenignSubmitError(err error) bool {
	if err == nil {
		return false
	}
	msg := strings.ToLower(err.Error())
	return strings.Contains(msg, "already known") ||
		strings.Contains(msg, "known transaction")
}

func (co *Connection) queryFailingError(ctx context.Context, hash common.Hash) error {
	client, err := co.liveClient()
	if err != nil {
		return err
	}
	tx, _, err := client.TransactionByHash(ctx, hash)
	if err != nil {
		return err
	}

	from, err := types.Sender(types.LatestSignerForChainID(tx.ChainId()), tx)
	if err != nil {
		return err
	}

	params := ethereum.CallMsg{
		From:     from,
		To:       tx.To(),
		Gas:      tx.Gas(),
		GasPrice: tx.GasPrice(),
		Value:    tx.Value(),
		Data:     tx.Data(),
	}

	log.WithFields(logrus.Fields{
		"From":     from,
		"To":       tx.To(),
		"Gas":      tx.Gas(),
		"GasPrice": tx.GasPrice(),
		"Value":    tx.Value(),
		"Data":     hex.EncodeToString(tx.Data()),
	}).Info("Call info")

	// The logger does a test call to the actual contract to check for any revert message and log it, as well
	// as logging the call info. This is because the golang client can sometimes suppress the log message and so
	// it can be helpful to use the call info to do the same call in Truffle/Web3js to get better logs.
	_, err = client.CallContract(ctx, params, nil)
	if err != nil {
		return err
	}
	return nil
}

const PollInterval uint64 = 12
const PendingTxTimeoutSecs uint64 = 180

func (co *Connection) waitForTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	var cnt uint64
	pendingTxTimeoutSecs := co.config.PendingTxTimeoutSecs
	if pendingTxTimeoutSecs == 0 {
		pendingTxTimeoutSecs = PendingTxTimeoutSecs
	}
	for {
		receipt, err := co.pollTransaction(ctx, tx, confirmations)
		if err != nil {
			return nil, err
		}

		if receipt != nil {
			return receipt, nil
		}

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(time.Duration(PollInterval) * time.Second):
			cnt++
			log.Info(fmt.Sprintf("waiting for receipt: %d seconds elapsed", cnt*PollInterval))
			if cnt*PollInterval > pendingTxTimeoutSecs {
				return nil, fmt.Errorf("wait receipt timeout")
			}
		}
	}
}

func (co *Connection) pollTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	receipt, err := co.Client().TransactionReceipt(ctx, tx.Hash())
	if err != nil {
		if errors.Is(err, goEthereum.NotFound) {
			return nil, nil
		}
	}

	latestHeader, err := co.Client().HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}

	if latestHeader != nil && receipt != nil && latestHeader.Number.Uint64()-receipt.BlockNumber.Uint64() >= confirmations {
		return receipt, nil
	}

	return nil, nil
}

func (co *Connection) WatchTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	receipt, err := co.waitForTransaction(ctx, tx, confirmations)
	if err != nil {
		return nil, err
	}
	if receipt.Status != 1 {
		err = co.queryFailingError(ctx, receipt.TxHash)
		logFields := log.Fields{
			"txHash": tx.Hash().Hex(),
		}
		if err != nil {
			logFields["error"] = err.Error()
			jsonErr, ok := err.(JsonError)
			if ok {
				errorCode := fmt.Sprintf("%v", jsonErr.ErrorData())
				logFields["code"] = errorCode
			}
		}
		log.WithFields(logFields).Error("Failed to send transaction")
		return receipt, err
	}
	return receipt, nil
}

func (co *Connection) MakeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := co.ChainID()
	keypair := co.Keypair()

	options := bind.TransactOpts{
		From: keypair.CommonAddress(),
		Signer: func(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
			return types.SignTx(tx, types.LatestSignerForChainID(chainID), keypair.PrivateKey())
		},
		Context: ctx,
	}

	if co.config.GasFeeCap > 0 {
		fee := big.NewInt(0)
		fee.SetUint64(co.config.GasFeeCap)
		options.GasFeeCap = fee
	}

	if co.config.GasTipCap > 0 {
		tip := big.NewInt(0)
		tip.SetUint64(co.config.GasTipCap)
		options.GasTipCap = tip
	}

	if co.config.GasLimit > 0 {
		options.GasLimit = co.config.GasLimit
	}

	return &options
}

func (co *Connection) WaitForFutureBlock(ctx context.Context, blockNumber uint64, gap uint64) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(time.Duration(PollInterval) * time.Second):
			latestHeader, err := co.Client().HeaderByNumber(ctx, nil)
			if err == nil && latestHeader != nil && latestHeader.Number.Uint64()-blockNumber >= gap {
				return nil
			}
		}
	}
}
