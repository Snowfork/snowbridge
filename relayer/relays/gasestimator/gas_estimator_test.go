package gasestimator

import (
	"context"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAssetHubAcceptedAndReward(t *testing.T) {
	ahOK := true
	ahFail := false
	est := makeEstimate(10, 4, big.NewInt(3), false)
	est.AssetHubDryRunSuccess = &ahOK
	est.ServiceFeeInEther = big.NewInt(2)
	assert.True(t, est.AssetHubAccepted())
	assert.Equal(t, big.NewInt(15), est.V2RewardWei(big.NewInt(10))) // 10 fee + 3 tip + 2 service

	est.AssetHubDryRunSuccess = &ahFail
	assert.False(t, est.AssetHubAccepted())

	// Older binaries omit the Asset Hub field; fall back to the combined flag.
	legacy := makeEstimate(1, 1, nil, true)
	assert.True(t, legacy.AssetHubAccepted())
	legacy.BridgeHub.DryRunSuccess = false
	assert.False(t, legacy.AssetHubAccepted())
}

func makeEstimate(extrinsicWei, deliveryWei int64, tipWei *big.Int, dryRunOK bool) *GasEstimate {
	est := &GasEstimate{}
	est.ExtrinsicFeeInEther = *big.NewInt(extrinsicWei)
	est.BridgeHub.DeliveryFeeInEther = *big.NewInt(deliveryWei)
	est.BridgeHub.TipInEther = tipWei
	est.BridgeHub.DryRunSuccess = dryRunOK
	return est
}

func makeEvent(relayerFeeWei *big.Int, blockNumber uint64, assets []contracts.Asset) *contracts.GatewayOutboundMessageAccepted {
	return &contracts.GatewayOutboundMessageAccepted{
		Nonce: 1,
		Payload: contracts.Payload{
			RelayerFee: relayerFeeWei,
			Assets:     assets,
		},
		Raw: ethtypes.Log{BlockNumber: blockNumber},
	}
}

// TestIsProfitableBaseFloor covers the gas break-even floor (relayer fee + tip + UI/service fee).
func TestIsProfitableBaseFloor(t *testing.T) {
	ctx := context.Background()

	t.Run("disabled accepts all", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: false}}
		require.NoError(t, g.IsProfitable(ctx, makeEstimate(10, 10, nil, true), makeEvent(big.NewInt(0), 0, nil), 0))
	})

	t.Run("dry run failure errors", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: true}}
		est := makeEstimate(10, 10, nil, false)
		msg := "boom"
		est.BridgeHub.DryRunError = &msg
		assert.Error(t, g.IsProfitable(ctx, est, makeEvent(big.NewInt(100), 0, nil), 0))
	})

	t.Run("reward below cost rejected", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: true}}
		err := g.IsProfitable(ctx, makeEstimate(60, 40, nil, true), makeEvent(big.NewInt(50), 0, nil), 0)
		assert.Error(t, err)
	})

	t.Run("reward equal to cost rejected", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: true}}
		assert.Error(t, g.IsProfitable(ctx, makeEstimate(60, 40, nil, true), makeEvent(big.NewInt(100), 0, nil), 0))
	})

	t.Run("reward above cost accepted", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: true}}
		require.NoError(t, g.IsProfitable(ctx, makeEstimate(60, 40, nil, true), makeEvent(big.NewInt(101), 0, nil), 0))
	})

	t.Run("tip counts toward reward", func(t *testing.T) {
		g := &Estimator{config: Config{Enabled: true}}
		// cost 100; relayer fee 60 + tip 50 = 110 > 100
		require.NoError(t, g.IsProfitable(ctx, makeEstimate(60, 40, big.NewInt(50), true), makeEvent(big.NewInt(60), 0, nil), 0))
	})
}

// nativeUSDCAsset builds a NativeTokenERC20 asset (Kind 0) carrying ABI-encoded (address, uint128).
func nativeUSDCAsset(t *testing.T, amount *big.Int) contracts.Asset {
	addressTy, err := abi.NewType("address", "", nil)
	require.NoError(t, err)
	uint128Ty, err := abi.NewType("uint128", "", nil)
	require.NoError(t, err)
	args := abi.Arguments{{Type: addressTy}, {Type: uint128Ty}}
	data, err := args.Pack(common.HexToAddress("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), amount)
	require.NoError(t, err)
	return contracts.Asset{Kind: 0, Data: data}
}

// TestIsProfitableServiceFee covers UI/service fee counting toward gas break-even
// (volume×rewardRate was removed).
func TestIsProfitableServiceFee(t *testing.T) {
	ctx := context.Background()
	g := &Estimator{config: Config{Enabled: true}}
	const cost = int64(1_000_000_000_000_000) // 0.001 ETH

	t.Run("service fee covers the shortfall", func(t *testing.T) {
		// relayer net alone is below cost; service fee pushes total over
		reward := big.NewInt(cost - 50_000_000_000_000)
		est := makeEstimate(cost, 0, nil, true)
		est.ServiceFeeInEther = big.NewInt(100_000_000_000_000)
		require.NoError(t, g.IsProfitable(ctx, est, makeEvent(reward, 0, nil), 0))
	})

	t.Run("service fee too small is still rejected", func(t *testing.T) {
		reward := big.NewInt(cost - 50_000_000_000_000)
		est := makeEstimate(cost, 0, nil, true)
		est.ServiceFeeInEther = big.NewInt(10_000_000_000_000)
		assert.Error(t, g.IsProfitable(ctx, est, makeEvent(reward, 0, nil), 0))
	})

	t.Run("assets alone do not gate without volume rate", func(t *testing.T) {
		assets := []contracts.Asset{nativeUSDCAsset(t, big.NewInt(1_000_000_000))}
		reward := big.NewInt(cost + 1)
		require.NoError(t, g.IsProfitable(ctx, makeEstimate(cost, 0, nil, true), makeEvent(reward, 0, assets), 0))
	})
}
