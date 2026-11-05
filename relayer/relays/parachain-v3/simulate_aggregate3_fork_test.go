package parachainV3

import (
	"context"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"golang.org/x/sync/errgroup"
)

// Default to the Multicall3 the parachain-v3 relayer batches through on mainnet
// (seen as multicall3Contract in the aggregate3 logs). Override with SNOWBRIDGE_FORK_MULTICALL3.
const defaultForkMulticall3 = "0xF57F9e622bc12DA707dB20a9e21Cd88F212efFC1"

// TestSimulateAggregate3AgainstFork is an opt-in integration harness that points
// simulateAggregate3's dry-run client at a forked-mainnet RPC (e.g.
// https://fork-mainnet.snowbridge.network) to prove the real eth_call dry-run executes.
//
// It is skipped unless SNOWBRIDGE_FORK_RPC_URL is set, so it never runs in normal CI. Run:
//
//	SNOWBRIDGE_FORK_RPC_URL=https://fork-mainnet.snowbridge.network \
//	SNOWBRIDGE_FORK_API_KEY=$FORKED_PROVIDER_API_KEY \
//	go test ./relays/parachain-v3/ -run TestSimulateAggregate3AgainstFork -v
func TestSimulateAggregate3AgainstFork(t *testing.T) {
	url := os.Getenv("SNOWBRIDGE_FORK_RPC_URL")
	if url == "" {
		t.Skip("set SNOWBRIDGE_FORK_RPC_URL to run the forked-mainnet dry-run harness")
	}
	apiKey := os.Getenv("SNOWBRIDGE_FORK_API_KEY")
	multicall3 := os.Getenv("SNOWBRIDGE_FORK_MULTICALL3")
	if multicall3 == "" {
		multicall3 = defaultForkMulticall3
	}

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	dryRunCfg := &config.EthereumConfig{Endpoint: url}
	if apiKey != "" {
		dryRunCfg.Headers = map[string]string{"X-API-Key": apiKey}
	}
	dryRunConn := ethereum.NewConnection(dryRunCfg, secp256k1.Alice())
	eg, egCtx := errgroup.WithContext(ctx)
	if err := dryRunConn.ConnectWithHeartBeat(egCtx, eg, 0); err != nil {
		t.Fatalf("connect dry-run conn to fork %s (unreachable or auth rejected): %v", url, err)
	}
	chainID := dryRunConn.ChainID()

	li := &BeefyInstantSyncer{
		config: &Config{
			Sink: SinkConfig{
				Contracts: SinkContractsConfig{Multicall3: multicall3},
			},
		},
		ethereumWriter: &EthereumWriter{conn: dryRunConn, dryRunConn: dryRunConn, readConn: dryRunConn},
	}

	mcABI, err := abi.JSON(strings.NewReader(contracts.Multicall3ABI))
	if err != nil {
		t.Fatalf("parse multicall3 ABI: %v", err)
	}
	innerCalldata, err := mcABI.Pack("getBlockNumber")
	if err != nil {
		t.Fatalf("pack getBlockNumber: %v", err)
	}
	calls := []contracts.Multicall3Call3{{
		Target:       common.HexToAddress(multicall3),
		AllowFailure: false,
		CallData:     innerCalldata,
	}}

	const gasLimit = uint64(2_000_000)
	metrics, err := li.simulateAggregate3(ctx, li.ethereumWriter.dryRunClientConn(), calls, gasLimit)
	if err != nil {
		t.Fatalf("simulateAggregate3 against fork failed: %v", err)
	}

	t.Logf("fork dry-run metrics: chainID=%s estimatedGas=%d gasPriceWei=%s estimatedFeeWei=%s conservativeFeeWei=%s",
		chainID, metrics.EstimatedGas, metrics.GasPriceWei, metrics.EstimatedFeeWei,
		metrics.ConservativeFeeWei)

	if metrics.EstimatedGas == 0 {
		t.Fatalf("expected a non-zero gas estimate from the fork")
	}
	if metrics.EstimatedFeeWei == nil || metrics.EstimatedFeeWei.Sign() <= 0 {
		t.Fatalf("expected a positive estimated fee, got %v", metrics.EstimatedFeeWei)
	}
	if metrics.ConservativeFeeWei == nil ||
		metrics.ConservativeFeeWei.Cmp(metrics.EstimatedFeeWei) < 0 {
		t.Fatalf("conservative fee %v should be >= estimated fee %v", metrics.ConservativeFeeWei, metrics.EstimatedFeeWei)
	}
}
