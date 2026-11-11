package parachainV3

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"math/big"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
)

func TestIsBenignBroadcastError(t *testing.T) {
	cases := []struct {
		err  error
		want bool
	}{
		{nil, false},
		{errors.New("replacement transaction underpriced"), false},
		{errors.New("insufficient funds for gas * price + value"), false},
		{errors.New("already known"), true},
		{errors.New("Already Known"), true},
		{errors.New("known transaction: 0xabc"), true},
		{errors.New("nonce too low"), true},
	}
	for _, tc := range cases {
		if got := isBenignBroadcastError(tc.err); got != tc.want {
			t.Fatalf("isBenignBroadcastError(%v)=%v, want %v", tc.err, got, tc.want)
		}
	}
}

func TestPrivateSubmitEndpointsFiltersEmpty(t *testing.T) {
	cfg := &Config{ExtraPrivateSubmitEndpoints: []string{"", "  ", "https://eu.rpc.titanbuilder.xyz", "\t"}}
	got := cfg.PrivateSubmitEndpoints()
	if len(got) != 1 || got[0] != "https://eu.rpc.titanbuilder.xyz" {
		t.Fatalf("PrivateSubmitEndpoints()=%v", got)
	}
	if len((&Config{}).PrivateSubmitEndpoints()) != 0 {
		t.Fatalf("empty config should yield no endpoints")
	}
}

func TestPrivateSubmitPathLabel(t *testing.T) {
	if got := privateSubmitPathLabel("https://eu.rpc.titanbuilder.xyz"); got != "titan" {
		t.Fatalf("titan label=%q", got)
	}
	if got := privateSubmitPathLabel("https://rpc.quasar.win"); got != "quasar" {
		t.Fatalf("quasar label=%q", got)
	}
	if got := privateSubmitPathLabel("https://rpc.example.com/path"); got != "rpc.example.com" {
		t.Fatalf("host label=%q", got)
	}
	if got := privateSubmitPathLabel("not a url"); got != "private-builder" {
		t.Fatalf("fallback label=%q", got)
	}
	if !privateSubmitUsesBundle("titan") || !privateSubmitUsesBundle("quasar") {
		t.Fatal("titan and quasar should use eth_sendBundle")
	}
	if privateSubmitUsesBundle("rpc.example.com") || privateSubmitUsesBundle("private-builder") {
		t.Fatal("other hosts should use eth_sendRawTransaction")
	}
}

func TestStaticAggregate3FeeGasOmitsBeefyHeadroom(t *testing.T) {
	msgGas := uint64(210_620) // estimatedGasForV2SubmitProof for unlock@BaseUnlockGas
	feeGas := staticAggregate3FeeGas(msgGas)
	want := BaseBeefyFiatShamirFeeGas + multicall3AggregateOverheadBuffer + msgGas
	if feeGas != want {
		t.Fatalf("staticAggregate3FeeGas=%d, want %d", feeGas, want)
	}
	// Fee pricing uses the calibrated fee constant, not the higher gas-limit floor component.
	if feeGas >= BaseBeefyFiatShamirGas+multicall3AggregateOverheadBuffer+msgGas {
		t.Fatalf("fee gas %d should be below limit-based BEEFY+overhead+msg", feeGas)
	}
	floor := multicallGasFloor(msgGas)
	if feeGas >= floor {
		t.Fatalf("fee gas %d should be below gas-limit floor %d (headroom omitted)", feeGas, floor)
	}
}

func TestStaticAggregate3FeeGasProfitableAgainstObservedRace(t *testing.T) {
	// Lost race nonce 727: single unlock, MaxDispatchGas=200k but fee prices BaseUnlockGas.
	proof := &MessageProof{}
	proof.Message.OriginalMessage.Commands = []CommandWrapper{
		{Kind: 2, MaxDispatchGas: types.NewU64(200_000)},
	}
	feeGas := staticAggregate3FeeGas(estimatedGasForV2SubmitProof(proof))
	baseFee := big.NewInt(61_410_779)
	collected := big.NewInt(158_187_382_210_000)
	estimated := new(big.Int).Mul(baseFee, new(big.Int).SetUint64(feeGas))
	if collected.Cmp(estimated) < 0 {
		t.Fatalf("static pricing would skip a profitable race: collected=%s estimated=%s feeGas=%d",
			collected, estimated, feeGas)
	}
	// Closer to actual burn (2,052,250) than MaxDispatchGas-based pricing (2,456,426).
	if feeGas >= 2_456_426 {
		t.Fatalf("feeGas %d still looks MaxDispatchGas-based", feeGas)
	}
}

func TestStaticAggregate3FeeGasProfitableAgainstNonce808(t *testing.T) {
	// Lost race nonce 808: old static feeGas (2.33M) × baseFee exceeded attached fee;
	// winning Multicall3 burned 2,075,820 gas. Calibrated FEE_GAS must clear raw break-even.
	proof := &MessageProof{}
	proof.Message.OriginalMessage.Commands = []CommandWrapper{
		{Kind: 2, MaxDispatchGas: types.NewU64(200_000)},
	}
	feeGas := staticAggregate3FeeGas(estimatedGasForV2SubmitProof(proof))
	baseFee := big.NewInt(164_858_306)           // from fastpath evaluation log
	collected := big.NewInt(366_588_028_620_000) // attachedFeeWei
	estimated := new(big.Int).Mul(baseFee, new(big.Int).SetUint64(feeGas))
	if collected.Cmp(estimated) < 0 {
		t.Fatalf("static pricing would still skip nonce 808: collected=%s estimated=%s feeGas=%d",
			collected, estimated, feeGas)
	}
	const actualGasUsed = uint64(2_075_820)
	if feeGas < actualGasUsed {
		t.Fatalf("feeGas %d below observed gasUsed %d (too aggressive)", feeGas, actualGasUsed)
	}
	// Must stay below the pre-calibration overestimate (~2.33M).
	legacyFeeGas := uint64(2_000_000) + multicall3AggregateOverheadBuffer + estimatedGasForV2SubmitProof(proof)
	if feeGas >= legacyFeeGas {
		t.Fatalf("feeGas %d did not improve on legacy %d", feeGas, legacyFeeGas)
	}
	// Thin (~2% over raw estimate): intentionally skipped once the 12.5% base-fee margin applies.
	basis := fastPathProfitabilityBasisFeeWei(estimated)
	if collected.Cmp(basis) >= 0 {
		t.Fatalf("nonce 808 should fail FastPathBaseFeeMarginBps gate: collected=%s basis=%s", collected, basis)
	}
}

func TestFastPathProfitabilityBasisFeeWeiDefaultMargin(t *testing.T) {
	if FastPathBaseFeeMarginBps != 1250 {
		t.Fatalf("default FastPathBaseFeeMarginBps=%d, want 1250 (12.5%%)", FastPathBaseFeeMarginBps)
	}
	estimated := big.NewInt(800)
	got := fastPathProfitabilityBasisFeeWei(estimated)
	// 800 * 11250 / 10000 = 900
	if got.Cmp(big.NewInt(900)) != 0 {
		t.Fatalf("basis=%s want 900", got)
	}
	if fastPathProfitabilityBasisFeeWei(nil) != nil {
		t.Fatalf("nil estimated should yield nil basis")
	}
}

func TestFastPathMarginKeepsComfortableRace(t *testing.T) {
	// Nonce 727 had ~18% slack over raw estimate — still clears the 12.5% gate.
	proof := &MessageProof{}
	proof.Message.OriginalMessage.Commands = []CommandWrapper{
		{Kind: 2, MaxDispatchGas: types.NewU64(200_000)},
	}
	feeGas := staticAggregate3FeeGas(estimatedGasForV2SubmitProof(proof))
	baseFee := big.NewInt(61_410_779)
	collected := big.NewInt(158_187_382_210_000)
	estimated := new(big.Int).Mul(baseFee, new(big.Int).SetUint64(feeGas))
	basis := fastPathProfitabilityBasisFeeWei(estimated)
	if collected.Cmp(basis) < 0 {
		t.Fatalf("comfortable race should pass margin gate: collected=%s basis=%s estimated=%s",
			collected, basis, estimated)
	}
}

func TestBroadcastSignedTxBundleToURL(t *testing.T) {
	key, err := crypto.GenerateKey()
	if err != nil {
		t.Fatal(err)
	}
	signer := gethTypes.LatestSignerForChainID(big.NewInt(1))
	tx := gethTypes.NewTx(&gethTypes.DynamicFeeTx{
		ChainID:   big.NewInt(1),
		Nonce:     1,
		GasTipCap: big.NewInt(1),
		GasFeeCap: big.NewInt(2),
		Gas:       21000,
		To:        &common.Address{},
		Value:     big.NewInt(0),
	})
	signed, err := gethTypes.SignTx(tx, signer, key)
	if err != nil {
		t.Fatal(err)
	}
	raw, err := signed.MarshalBinary()
	if err != nil {
		t.Fatal(err)
	}
	wantRaw := hexutil.Encode(raw)

	var sawMethod string
	var sawParams []json.RawMessage
	var sawSignature string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sawSignature = r.Header.Get("X-Flashbots-Signature")
		body, _ := io.ReadAll(r.Body)
		var req struct {
			Method string            `json:"method"`
			Params []json.RawMessage `json:"params"`
		}
		if err := json.Unmarshal(body, &req); err != nil {
			http.Error(w, err.Error(), 400)
			return
		}
		switch req.Method {
		case "eth_chainId":
			_, _ = w.Write([]byte(`{"jsonrpc":"2.0","id":1,"result":"0x1"}`))
		case "eth_sendBundle":
			sawMethod = req.Method
			sawParams = req.Params
			_, _ = w.Write([]byte(`{"jsonrpc":"2.0","id":1,"result":{"bundleHash":"0xabc"}}`))
		default:
			http.Error(w, "unexpected "+req.Method, 500)
		}
	}))
	defer srv.Close()

	if err := broadcastSignedTxBundleToURL(context.Background(), srv.URL, signed, "titan", nil, nil); err != nil {
		t.Fatalf("broadcastSignedTxBundleToURL: %v", err)
	}
	if sawMethod != "eth_sendBundle" {
		t.Fatalf("method=%q", sawMethod)
	}
	if len(sawParams) != 1 {
		t.Fatalf("params len=%d", len(sawParams))
	}
	var bundle struct {
		Txs               []string `json:"txs"`
		BlockNumber       string   `json:"blockNumber"`
		RevertingTxHashes []string `json:"revertingTxHashes"`
	}
	if err := json.Unmarshal(sawParams[0], &bundle); err != nil {
		t.Fatalf("unmarshal bundle params: %v", err)
	}
	if len(bundle.Txs) != 1 || bundle.Txs[0] != wantRaw {
		t.Fatalf("txs=%v want [%s]", bundle.Txs, wantRaw)
	}
	if bundle.BlockNumber != "" {
		t.Fatalf("blockNumber should be omitted, got %q", bundle.BlockNumber)
	}
	if bundle.RevertingTxHashes != nil {
		t.Fatalf("revertingTxHashes should be omitted, got %v", bundle.RevertingTxHashes)
	}
	if sawSignature != "" {
		t.Fatalf("titan request should not set X-Flashbots-Signature, got %q", sawSignature)
	}
}

func TestBroadcastQuasarBundleSignsRequest(t *testing.T) {
	kp, err := secp256k1.GenerateKeypair()
	if err != nil {
		t.Fatal(err)
	}
	key := kp.PrivateKey()
	signer := gethTypes.LatestSignerForChainID(big.NewInt(1))
	tx := gethTypes.NewTx(&gethTypes.DynamicFeeTx{
		ChainID:   big.NewInt(1),
		Nonce:     7,
		GasTipCap: big.NewInt(1),
		GasFeeCap: big.NewInt(2),
		Gas:       21000,
		To:        &common.Address{},
		Value:     big.NewInt(0),
	})
	signed, err := gethTypes.SignTx(tx, signer, key)
	if err != nil {
		t.Fatal(err)
	}

	var body []byte
	var header string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		header = r.Header.Get("X-Flashbots-Signature")
		body, _ = io.ReadAll(r.Body)
		_, _ = w.Write([]byte(`{"jsonrpc":"2.0","id":1,"result":{"bundleHash":"0xabc"}}`))
	}))
	defer srv.Close()

	if err := broadcastSignedTxBundleToURL(context.Background(), srv.URL, signed, "quasar", nil, kp); err != nil {
		t.Fatalf("quasar bundle: %v", err)
	}
	if header == "" || !strings.HasPrefix(header, kp.CommonAddress().Hex()+":") {
		t.Fatalf("header=%q", header)
	}
	want, err := flashbotsSignatureHeader(body, kp)
	if err != nil {
		t.Fatal(err)
	}
	if header != want {
		t.Fatalf("header does not sign the posted body")
	}
	sigHex := strings.TrimPrefix(header, kp.CommonAddress().Hex()+":")
	sig, err := hexutil.Decode(sigHex)
	if err != nil {
		t.Fatal(err)
	}
	if sig[64] < 27 {
		t.Fatalf("signature v=%d, want 27/28", sig[64])
	}
	sig[64] -= 27
	hashedBody := crypto.Keccak256Hash(body).Hex()
	pub, err := crypto.SigToPub(accounts.TextHash([]byte(hashedBody)), sig)
	if err != nil {
		t.Fatal(err)
	}
	if crypto.PubkeyToAddress(*pub) != kp.CommonAddress() {
		t.Fatal("recovered address does not match the fast-path key")
	}
}

func TestShouldRebundleTitanOnHeadAdvance(t *testing.T) {
	cases := []struct {
		head, last uint64
		want       bool
	}{
		{100, 100, false},
		{101, 100, true},
		{105, 100, true},
		{99, 100, false},
		{1, 0, true},
		{0, 0, false},
	}
	for _, tc := range cases {
		if got := shouldRebundleTitan(tc.head, tc.last); got != tc.want {
			t.Fatalf("shouldRebundleTitan(%d,%d)=%v want %v", tc.head, tc.last, got, tc.want)
		}
	}
}
