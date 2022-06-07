package ethereum

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/config"
)

const (
	DefaultMaxTip = 25_000_000_000  // 25 Gwei if no TipCap is specified in config.
	DefaultMaxGas = 300_000_000_000 // 300 Gwei if no GasCap is specified in config.
)

type FeeInfo struct {
	GasTipCap *big.Int
	GasFeeCap *big.Int
}

func scaleFee(multiplier float64, suggested *big.Int, max, defaultMax uint64) *big.Int {
	multiplied, _ := new(big.Float).Mul(big.NewFloat(multiplier), new(big.Float).SetInt(suggested)).Int(nil)
	maxValue := big.NewInt(int64(defaultMax))
	if max > 0 {
		maxValue = new(big.Int).SetUint64(max)
	}
	if multiplied.Cmp(suggested) < 0 {
		return suggested
	}
	if multiplied.Cmp(maxValue) > 0 {
		return maxValue
	}
	return multiplied
}

func CalculateFee(config config.EthereumConfig, suggestedBase, suggestedTip *big.Int) FeeInfo {
	result := FeeInfo{}

	if suggestedTip != nil && config.GasTipMultiplier != 0 {
		result.GasTipCap = scaleFee(config.GasTipMultiplier, suggestedTip, config.GasTipCap, DefaultMaxTip)
	}

	if suggestedBase != nil && config.GasFeeMultiplier != 0 {
		result.GasFeeCap = scaleFee(config.GasFeeMultiplier, suggestedBase, config.GasFeeCap, DefaultMaxGas)
	}

	if config.GasTipCap > 0 && result.GasTipCap == nil {
		tip := big.NewInt(0)
		tip.SetUint64(config.GasTipCap)
		result.GasTipCap = tip
	}

	if config.GasFeeCap > 0 && result.GasFeeCap == nil {
		fee := big.NewInt(0)
		fee.SetUint64(config.GasFeeCap)
		result.GasFeeCap = fee
	}

	log.WithFields(log.Fields{
		"gasFeeMultiplier": config.GasFeeMultiplier,
		"gasTipMultiplier": config.GasTipMultiplier,
		"gasFeeCap":        config.GasFeeCap,
		"gasTipCap":        config.GasTipCap,
		"suggestedFee":     suggestedBase,
		"suggestedCap":     suggestedTip,
		"chosenFee":        result.GasFeeCap,
		"chosenTip":        result.GasTipCap,
	}).Debug("Transaction Fees")

	return result
}

func MakeTxOpts(conn *Connection, config config.EthereumConfig, ctx context.Context) (*bind.TransactOpts, error) {
	chainID := conn.ChainID()
	keypair := conn.Keypair()

	options := bind.TransactOpts{
		From: keypair.CommonAddress(),
		Signer: func(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
			return types.SignTx(tx, types.NewLondonSigner(chainID), keypair.PrivateKey())
		},
		Context: ctx,
	}

	var suggestedBase, suggestedTip *big.Int
	if config.GasTipMultiplier > 0 {
		if tip, err := conn.Client().SuggestGasTipCap(ctx); err == nil {
			suggestedTip = tip
		} else {
			return nil, err
		}
	}
	if config.GasFeeMultiplier > 0 {
		if head, err := conn.Client().HeaderByNumber(ctx, nil); err == nil {
			suggestedBase = head.BaseFee
		} else {
			return nil, err
		}
	}

	feeInfo := CalculateFee(config, suggestedBase, suggestedTip)
	options.GasFeeCap = feeInfo.GasFeeCap
	options.GasTipCap = feeInfo.GasTipCap

	if config.GasLimit > 0 {
		options.GasLimit = config.GasLimit
	}

	return &options, nil
}
