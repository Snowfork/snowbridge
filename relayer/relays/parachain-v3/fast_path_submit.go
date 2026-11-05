package parachainV3

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	gethcrypto "github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
)

// fastPathProfitabilityBasisFeeWei inflates estimated execution cost by
// FastPathBaseFeeMarginBps (default 12.5% = one EIP-1559 max base-fee uptick) so a
// typical next-block inclusion still break-even if base fee rises fully.
func fastPathProfitabilityBasisFeeWei(estimatedFeeWei *big.Int) *big.Int {
	if estimatedFeeWei == nil {
		return nil
	}
	// basis = estimated * (10000 + bps) / 10000
	scaled := new(big.Int).Mul(estimatedFeeWei, new(big.Int).SetUint64(10_000+FastPathBaseFeeMarginBps))
	return scaled.Div(scaled, big.NewInt(10_000))
}

// evaluateStaticFiatShamirProfitability prices the aggregate3 batch at live mainnet base
// fee × static fee gas (no fork eth_call / eth_estimateGas). Profitability requires
// collected fee ≥ estimated cost after FastPathBaseFeeMarginBps.
func (li *BeefyInstantSyncer) evaluateStaticFiatShamirProfitability(
	ctx context.Context,
	minSubmitProofGasSum uint64,
	collectedFeeWei *big.Int,
	gasLimit uint64,
) (*DryRunMetrics, bool, error) {
	liveHeader, err := li.ethereumWriter.readConn.Client().HeaderByNumber(ctx, nil)
	if err != nil || liveHeader == nil || liveHeader.BaseFee == nil {
		return nil, false, fmt.Errorf("fetch live base fee for static fast-path fee estimate: %w", err)
	}
	baseFeeWei := liveHeader.BaseFee
	feeGas := staticAggregate3FeeGas(minSubmitProofGasSum)
	estimatedFeeWei := new(big.Int).Mul(baseFeeWei, new(big.Int).SetUint64(feeGas))
	conservativeFeeWei := new(big.Int).Mul(baseFeeWei, new(big.Int).SetUint64(gasLimit))
	metrics := &DryRunMetrics{
		EstimatedGas:       feeGas,
		GasPriceWei:        baseFeeWei,
		EstimatedFeeWei:    estimatedFeeWei,
		ConservativeFeeWei: conservativeFeeWei,
		EffectiveGasLimit:  gasLimit,
	}
	basisFeeWei := fastPathProfitabilityBasisFeeWei(estimatedFeeWei)
	profitable := collectedFeeWei != nil && basisFeeWei != nil && collectedFeeWei.Cmp(basisFeeWei) >= 0
	return metrics, profitable, nil
}

// submitAggregate3FastPath signs once under submitMu (nonce-safe), broadcasts the same
// raw tx to Flashbots and optional private builders, then waits for the receipt outside
// the lock so catch-up / handover / instant can interleave.
//
// While waiting, Titan and Quasar eth_sendBundle are re-submitted whenever the public head
// advances: both target only the current block by default and do not auto-retry missed bundles.
//
// Public mempool dual-submit is intentionally omitted: after a lost race, public inclusion
// of a stale BEEFY+message tx can land a paid revert; private builders drop simulated
// reverts instead.
func (li *BeefyInstantSyncer) submitAggregate3FastPath(
	ctx context.Context,
	txOpts *bind.TransactOpts,
	calls []contracts.Multicall3Call3,
) (*gethTypes.Transaction, error) {
	tx, bundleEndpoints, bundleSigner, err := li.broadcastAggregate3FastPath(ctx, txOpts, calls)
	if err != nil {
		return nil, err
	}

	watchConn := li.ethereumWriter.readConn
	if watchConn == nil || watchConn.Client() == nil {
		watchConn = li.ethereumWriter.conn
	}

	waitCtx, cancel := context.WithCancel(ctx)
	defer cancel()
	if len(bundleEndpoints) > 0 && watchConn != nil && watchConn.Client() != nil {
		go rebundleTitanOnNewBlocks(waitCtx, watchConn.Client(), tx, bundleEndpoints, bundleSigner)
	}

	receipt, err := watchConn.WatchTransaction(ctx, tx, 1)
	cancel()
	if err != nil {
		return tx, fmt.Errorf("watch multicall3 aggregate3 transaction: %w", err)
	}
	if err := li.logMulticallReceipt(receipt); err != nil {
		return tx, fmt.Errorf("parse multicall receipt: %w", err)
	}
	return tx, nil
}

// broadcastAggregate3FastPath builds+signs+broadcasts under submitMu. Caller waits outside.
func (li *BeefyInstantSyncer) broadcastAggregate3FastPath(
	ctx context.Context,
	txOpts *bind.TransactOpts,
	calls []contracts.Multicall3Call3,
) (*gethTypes.Transaction, []string, *secp256k1.Keypair, error) {
	li.submitMu.Lock()
	defer li.submitMu.Unlock()

	txOpts.NoSend = true
	tx, err := li.multicall3.Aggregate3(txOpts, calls)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("build multicall3 aggregate3 transaction: %w", err)
	}

	if err := broadcastSignedTx(ctx, li.ethereumWriter.conn, tx, "flashbots"); err != nil {
		return nil, nil, nil, fmt.Errorf("submit multicall3 aggregate3 via Flashbots: %w", err)
	}

	bundleSigner := li.ethereumWriter.conn.Keypair()
	var bundleEndpoints []string
	for _, endpoint := range li.config.PrivateSubmitEndpoints() {
		path := privateSubmitPathLabel(endpoint)
		var berr error
		if privateSubmitUsesBundle(path) {
			bundleEndpoints = append(bundleEndpoints, endpoint)
			berr = broadcastSignedTxBundleToURL(ctx, endpoint, tx, path, nil, bundleSigner)
		} else {
			berr = broadcastSignedTxToURL(ctx, endpoint, tx, path)
		}
		if berr != nil {
			log.WithError(berr).WithFields(log.Fields{
				"txHash":   tx.Hash().Hex(),
				"path":     path,
				"endpoint": endpoint,
			}).Warn("[instant-v2] Extra private dual-submit failed; continuing with Flashbots wait")
		}
	}
	return tx, bundleEndpoints, bundleSigner, nil
}

// titanRebundlePoll is how often we sample public head for Titan/Quasar eth_sendBundle refresh.
// Kept shorter than ethereum.PollInterval (receipt wait): builder auctions close within a
// ~12s slot, so a 12s rebundle tick often joins late. 3s is early enough without hammering RPC.
const titanRebundlePoll = 3 * time.Second

type headerByNumber interface {
	HeaderByNumber(ctx context.Context, number *big.Int) (*gethTypes.Header, error)
}

type transactionSender interface {
	SendTransaction(ctx context.Context, tx *gethTypes.Transaction) error
}

// shouldRebundleTitan reports whether head has advanced past the block we last bundled for.
func shouldRebundleTitan(headBlock, lastBundleBlock uint64) bool {
	return headBlock > lastBundleBlock
}

// rebundleTitanOnNewBlocks re-sends the same signed tx via eth_sendBundle whenever the
// public head advances, until ctx is cancelled (receipt seen or wait aborted).
// Titan and Quasar both default the bundle to the current block and do not carry it forward.
func rebundleTitanOnNewBlocks(
	ctx context.Context,
	headerSource headerByNumber,
	tx *gethTypes.Transaction,
	endpoints []string,
	signer *secp256k1.Keypair,
) {
	if headerSource == nil || tx == nil || len(endpoints) == 0 {
		return
	}
	var lastBundleBlock uint64
	if header, err := headerSource.HeaderByNumber(ctx, nil); err == nil && header != nil && header.Number != nil {
		// Seed to current head so we do not immediately duplicate the initial submit slot.
		lastBundleBlock = header.Number.Uint64()
	}

	ticker := time.NewTicker(titanRebundlePoll)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			header, err := headerSource.HeaderByNumber(ctx, nil)
			if err != nil || header == nil || header.Number == nil {
				continue
			}
			head := header.Number.Uint64()
			if !shouldRebundleTitan(head, lastBundleBlock) {
				continue
			}
			extra := log.Fields{
				"rebundle":  true,
				"headBlock": head,
			}
			for _, endpoint := range endpoints {
				path := privateSubmitPathLabel(endpoint)
				if err := broadcastSignedTxBundleToURL(ctx, endpoint, tx, path, extra, signer); err != nil {
					if ctx.Err() != nil {
						return
					}
					log.WithError(err).WithFields(log.Fields{
						"txHash":    tx.Hash().Hex(),
						"path":      path,
						"endpoint":  endpoint,
						"rebundle":  true,
						"headBlock": head,
					}).Warn("[instant-v2] Private bundle rebundle failed; continuing wait")
				}
			}
			lastBundleBlock = head
		}
	}
}

// broadcastSignedTx injects a pre-signed transaction into the given connection's pool.
// Benign "already known" responses (e.g. second path after the first) are treated as success.
func broadcastSignedTx(ctx context.Context, conn *ethereum.Connection, tx *gethTypes.Transaction, path string) error {
	if conn == nil || conn.Client() == nil {
		return fmt.Errorf("%s connection unavailable", path)
	}
	return sendSignedTx(ctx, conn.Client(), tx, path)
}

// broadcastSignedTxToURL dials a submit-only HTTP/WS endpoint (no NetworkID/heartbeat) and
// sends the signed tx. Used for builder RPCs like Titan that reject eth_chainId.
func broadcastSignedTxToURL(ctx context.Context, endpoint string, tx *gethTypes.Transaction, path string) error {
	if strings.TrimSpace(endpoint) == "" {
		return fmt.Errorf("%s endpoint empty", path)
	}
	client, err := ethclient.DialContext(ctx, endpoint)
	if err != nil {
		return fmt.Errorf("dial %s: %w", path, err)
	}
	defer client.Close()
	return sendSignedTx(ctx, client, tx, path)
}

func sendSignedTx(ctx context.Context, client transactionSender, tx *gethTypes.Transaction, path string) error {
	if client == nil {
		return fmt.Errorf("%s client unavailable", path)
	}
	if err := client.SendTransaction(ctx, tx); err != nil {
		if isBenignBroadcastError(err) {
			log.WithFields(log.Fields{
				"txHash": tx.Hash().Hex(),
				"path":   path,
				"error":  err.Error(),
			}).Info("[instant-v2] Broadcast already known on path")
			return nil
		}
		return err
	}
	log.WithFields(log.Fields{
		"txHash": tx.Hash().Hex(),
		"path":   path,
	}).Info("[instant-v2] Broadcast aggregate3 transaction")
	return nil
}

// titanSendBundleResult is the eth_sendBundle success payload from Titan.
type titanSendBundleResult struct {
	BundleHash string `json:"bundleHash"`
}

// broadcastSignedTxBundleToURL submits a single-tx eth_sendBundle (no blockNumber,
// no revertingTxHashes) so the builder drops on simulation revert instead of mining a paid revert.
// extraFields are merged into the success log (e.g. rebundle/headBlock during wait).
// Quasar requires an X-Flashbots-Signature over this request body; Titan does not.
func broadcastSignedTxBundleToURL(
	ctx context.Context,
	endpoint string,
	tx *gethTypes.Transaction,
	path string,
	extraFields log.Fields,
	signer *secp256k1.Keypair,
) error {
	if strings.TrimSpace(endpoint) == "" {
		return fmt.Errorf("%s endpoint empty", path)
	}
	raw, err := tx.MarshalBinary()
	if err != nil {
		return fmt.Errorf("marshal signed tx for %s bundle: %w", path, err)
	}
	params := sendBundleParams{Txs: []string{hexutil.Encode(raw)}}

	var result titanSendBundleResult
	if path == "quasar" {
		result, err = postSignedQuasarBundle(ctx, endpoint, params, signer)
	} else {
		result, err = postBundleViaEthClient(ctx, endpoint, path, params)
	}
	if err != nil {
		return err
	}
	fields := log.Fields{
		"txHash":     tx.Hash().Hex(),
		"path":       path,
		"method":     "eth_sendBundle",
		"bundleHash": result.BundleHash,
	}
	for k, v := range extraFields {
		fields[k] = v
	}
	log.WithFields(fields).Info("[instant-v2] Broadcast aggregate3 bundle")
	return nil
}

type sendBundleParams struct {
	Txs []string `json:"txs"`
}

type sendBundleRequest struct {
	JSONRPC string             `json:"jsonrpc"`
	ID      int                `json:"id"`
	Method  string             `json:"method"`
	Params  []sendBundleParams `json:"params"`
}

func postBundleViaEthClient(ctx context.Context, endpoint, path string, params sendBundleParams) (titanSendBundleResult, error) {
	client, err := ethclient.DialContext(ctx, endpoint)
	if err != nil {
		return titanSendBundleResult{}, fmt.Errorf("dial %s: %w", path, err)
	}
	defer client.Close()
	var result titanSendBundleResult
	if err := client.Client().CallContext(ctx, &result, "eth_sendBundle", params); err != nil {
		return titanSendBundleResult{}, err
	}
	return result, nil
}

// postSignedQuasarBundle POSTs eth_sendBundle and sets X-Flashbots-Signature to
// address:personal_sign(keccak256(body)). The signature is over the request, not a spend.
func postSignedQuasarBundle(ctx context.Context, endpoint string, params sendBundleParams, signer *secp256k1.Keypair) (titanSendBundleResult, error) {
	if signer == nil || signer.PrivateKey() == nil {
		return titanSendBundleResult{}, fmt.Errorf("quasar bundle signature key unavailable")
	}
	body, err := json.Marshal(sendBundleRequest{
		JSONRPC: "2.0",
		ID:      1,
		Method:  "eth_sendBundle",
		Params:  []sendBundleParams{params},
	})
	if err != nil {
		return titanSendBundleResult{}, fmt.Errorf("marshal quasar bundle: %w", err)
	}
	header, err := flashbotsSignatureHeader(body, signer)
	if err != nil {
		return titanSendBundleResult{}, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, endpoint, bytes.NewReader(body))
	if err != nil {
		return titanSendBundleResult{}, fmt.Errorf("build quasar request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Flashbots-Signature", header)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return titanSendBundleResult{}, err
	}
	defer resp.Body.Close()
	respBody, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	if err != nil {
		return titanSendBundleResult{}, fmt.Errorf("read quasar response: %w", err)
	}
	if resp.StatusCode != http.StatusOK {
		return titanSendBundleResult{}, fmt.Errorf("quasar status %d: %s", resp.StatusCode, strings.TrimSpace(string(respBody)))
	}
	var parsed struct {
		Result titanSendBundleResult `json:"result"`
		Error  json.RawMessage       `json:"error"`
	}
	if err := json.Unmarshal(respBody, &parsed); err != nil {
		return titanSendBundleResult{}, fmt.Errorf("decode quasar response: %w", err)
	}
	if len(parsed.Error) > 0 && string(parsed.Error) != "null" {
		return titanSendBundleResult{}, fmt.Errorf("quasar error: %s", strings.TrimSpace(string(parsed.Error)))
	}
	return parsed.Result, nil
}

// flashbotsSignatureHeader is the Flashbots auth header: checksummed address, then an
// EIP-191 personal_sign of the keccak256 hex of the exact request body.
func flashbotsSignatureHeader(body []byte, signer *secp256k1.Keypair) (string, error) {
	hashedBody := gethcrypto.Keccak256Hash(body).Hex()
	sig, err := gethcrypto.Sign(accounts.TextHash([]byte(hashedBody)), signer.PrivateKey())
	if err != nil {
		return "", fmt.Errorf("sign quasar bundle request: %w", err)
	}
	sig[64] += 27
	return signer.CommonAddress().Hex() + ":" + hexutil.Encode(sig), nil
}

func isBenignBroadcastError(err error) bool {
	if err == nil {
		return false
	}
	msg := strings.ToLower(err.Error())
	return strings.Contains(msg, "already known") ||
		strings.Contains(msg, "known transaction") ||
		strings.Contains(msg, "nonce too low")
}

// privateSubmitUsesBundle is true for builders that accept eth_sendBundle and target only
// the current block unless the bundle is refreshed. Other hosts get eth_sendRawTransaction.
func privateSubmitUsesBundle(path string) bool {
	switch path {
	case "titan", "quasar":
		return true
	default:
		return false
	}
}

func privateSubmitPathLabel(endpoint string) string {
	u, err := url.Parse(endpoint)
	if err != nil || u.Host == "" {
		return "private-builder"
	}
	host := strings.ToLower(u.Host)
	switch {
	case strings.Contains(host, "titanbuilder"):
		return "titan"
	case strings.Contains(host, "quasar.win"):
		return "quasar"
	default:
		return host
	}
}
