// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum"
	goEthereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"golang.org/x/sync/errgroup"
)

type Connection struct {
	endpoint       string
	kp             *secp256k1.Keypair
	client         *ethclient.Client
	fallbackClient *ethclient.Client
	chainID        *big.Int
	config         *config.EthereumConfig
}

type JsonError interface {
	Error() string
	ErrorCode() int
	ErrorData() interface{}
}

func NewConnection(config *config.EthereumConfig, kp *secp256k1.Keypair) *Connection {
	return &Connection{
		endpoint: config.Endpoint,
		kp:       kp,
		config:   config,
	}
}

func (co *Connection) ConnectWithHeartBeat(ctx context.Context, eg *errgroup.Group, heartBeat time.Duration) error {
	client, err := ethclient.Dial(co.endpoint)
	if err != nil {
		return err
	}

	chainID, err := client.NetworkID(ctx)
	if err != nil {
		return err
	}

	log.WithFields(logrus.Fields{
		"endpoint": co.endpoint,
		"chainID":  chainID,
	}).Info("Connected to chain")

	co.client = client
	co.chainID = chainID

	if co.config.FallbackEndpoint != "" {
		fb, err := ethclient.Dial(co.config.FallbackEndpoint)
		if err != nil {
			return fmt.Errorf("dial fallback Ethereum endpoint: %w", err)
		}
		fbChainID, err := fb.NetworkID(ctx)
		if err != nil {
			fb.Close()
			return fmt.Errorf("fallback endpoint network id: %w", err)
		}
		if fbChainID.Cmp(chainID) != 0 {
			fb.Close()
			return fmt.Errorf("fallback endpoint chain ID %s != primary %s", fbChainID.String(), chainID.String())
		}
		co.fallbackClient = fb
		log.WithFields(logrus.Fields{
			"fallbackEndpoint": co.config.FallbackEndpoint,
			"chainID":          fbChainID,
		}).Info("Connected fallback Ethereum RPC")
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
					_, err := client.NetworkID(ctx)
					if err != nil {
						log.WithField("endpoint", co.endpoint).Error("Connection heartbeat failed")
					}
				}
			}
		})
	}

	return nil
}

func (co *Connection) Close() {
	if co.client != nil {
		co.client.Close()
	}
	if co.fallbackClient != nil {
		co.fallbackClient.Close()
	}
}

func (co *Connection) Client() *ethclient.Client {
	return co.client
}

// FallbackClient returns the optional second RPC client, or nil if not configured.
func (co *Connection) FallbackClient() *ethclient.Client {
	return co.fallbackClient
}

func (co *Connection) Keypair() *secp256k1.Keypair {
	return co.kp
}

func (co *Connection) ChainID() *big.Int {
	return co.chainID
}

func (co *Connection) queryFailingErrorWithClient(ctx context.Context, client *ethclient.Client, hash common.Hash) error {
	if client == nil {
		client = co.client
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

// eip1559BaseFeeWiggleMultiplier matches go-ethereum (const basefeeWiggleMultiplier = 2 in accounts/abi/bind/v2/base.go).
// With EIP-1559, baseFee increases are capped at 12.5% per block in the worst case, so 2x baseFee gives enough slack for several blocks even under max increase (roughly ~6 blocks of worst-case growth).
const eip1559BaseFeeWiggleMultiplier int64 = 2

func (co *Connection) waitForTransactionWithClient(ctx context.Context, client *ethclient.Client, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	if client == nil {
		client = co.client
	}
	var cnt uint64
	for {
		receipt, err := co.pollTransactionWithClient(ctx, client, tx, confirmations)
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
			if co.config.PendingTxTimeoutSecs > 0 {
				cnt++
				log.Info(fmt.Sprintf("waiting for receipt: %d seconds elapsed", cnt*PollInterval))
				if cnt*PollInterval > co.config.PendingTxTimeoutSecs {
					return nil, fmt.Errorf("wait receipt timeout")
				}
			}
		}
	}
}

func (co *Connection) pollTransactionWithClient(ctx context.Context, client *ethclient.Client, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	if client == nil {
		client = co.client
	}
	receipt, err := client.TransactionReceipt(ctx, tx.Hash())
	if err != nil {
		if errors.Is(err, goEthereum.NotFound) {
			return nil, nil
		}
	}

	latestHeader, err := client.HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}

	if latestHeader != nil && receipt != nil && latestHeader.Number.Uint64()-receipt.BlockNumber.Uint64() >= confirmations {
		return receipt, nil
	}

	return nil, nil
}

func (co *Connection) WatchTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	return co.WatchTransactionWithClient(ctx, co.client, tx, confirmations)
}

// WatchTransactionWithClient waits for a receipt using the given RPC client (e.g. same endpoint used to broadcast).
func (co *Connection) WatchTransactionWithClient(ctx context.Context, client *ethclient.Client, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	if client == nil {
		client = co.client
	}
	receipt, err := co.waitForTransactionWithClient(ctx, client, tx, confirmations)
	if err != nil {
		return nil, err
	}
	if receipt.Status != 1 {
		err = co.queryFailingErrorWithClient(ctx, client, receipt.TxHash)
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

// gasFeeBumpScale returns numerator/denominator for scaling suggested caps.
// Both values must be configured (non-zero), otherwise ApplySuggestedGasFees will fail.
func (co *Connection) gasFeeBumpScale() (num, den uint64, err error) {
	num, den = co.config.GasFeeBumpNumerator, co.config.GasFeeBumpDenominator
	if num == 0 || den == 0 {
		return 0, 0, errors.New("ethereum config: gas-fee-bump-numerator/denominator must both be set and non-zero")
	}
	return num, den, nil
}

// suggestedEIP1559GasCaps returns maxPriorityFeePerGas and maxFeePerGas like go-ethereum bind createDynamicTx,
// then scales both by the configured bump ratio.
func (co *Connection) suggestedEIP1559GasCaps(ctx context.Context) (*big.Int, *big.Int, error) {
	head, err := co.client.HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, nil, fmt.Errorf("latest header: %w", err)
	}
	if head.BaseFee == nil {
		return nil, nil, errors.New("latest block has no base fee (pre-EIP-1559 chain)")
	}
	tip, err := co.client.SuggestGasTipCap(ctx)
	if err != nil {
		return nil, nil, fmt.Errorf("suggest gas tip cap: %w", err)
	}
	feeCap := new(big.Int).Add(
		tip,
		new(big.Int).Mul(head.BaseFee, big.NewInt(eip1559BaseFeeWiggleMultiplier)),
	)

	num, den, err := co.gasFeeBumpScale()
	if err != nil {
		return nil, nil, err
	}
	numBig := new(big.Int).SetUint64(num)
	denBig := new(big.Int).SetUint64(den)

	tipOut := new(big.Int).Mul(tip, numBig)
	tipOut.Div(tipOut, denBig)

	feeOut := new(big.Int).Mul(feeCap, numBig)
	feeOut.Div(feeOut, denBig)

	if feeOut.Cmp(tipOut) < 0 {
		feeOut = new(big.Int).Set(tipOut)
	}
	return tipOut, feeOut, nil
}

// ApplySuggestedGasFees sets opts.GasTipCap and opts.GasFeeCap from the RPC when static caps are not configured.
func (co *Connection) ApplySuggestedGasFees(ctx context.Context, opts *bind.TransactOpts) error {
	if co.config.GasFeeCap > 0 && co.config.GasTipCap > 0 {
		return nil
	}
	if co.config.GasFeeCap > 0 || co.config.GasTipCap > 0 {
		return errors.New("ethereum config: set both gas-fee-cap and gas-tip-cap, or omit both for dynamic EIP-1559 pricing")
	}
	tip, fee, err := co.suggestedEIP1559GasCaps(ctx)
	if err != nil {
		return err
	}
	opts.GasTipCap = tip
	opts.GasFeeCap = fee
	return nil
}

func (co *Connection) MakeTxOpts(ctx context.Context) (*bind.TransactOpts, error) {
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
		fee := new(big.Int).SetUint64(co.config.GasFeeCap)
		options.GasFeeCap = fee
	}

	if co.config.GasTipCap > 0 {
		tip := new(big.Int).SetUint64(co.config.GasTipCap)
		options.GasTipCap = tip
	}

	if co.config.GasLimit > 0 {
		options.GasLimit = co.config.GasLimit
	}

	if co.config.GasFeeCap == 0 && co.config.GasTipCap == 0 {
		if err := co.ApplySuggestedGasFees(ctx, &options); err != nil {
			return nil, fmt.Errorf("dynamic gas fees: %w", err)
		}
	} else if co.config.GasFeeCap == 0 || co.config.GasTipCap == 0 {
		return nil, errors.New("ethereum config: set both gas-fee-cap and gas-tip-cap, or omit both for dynamic EIP-1559 pricing")
	}

	return &options, nil
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
