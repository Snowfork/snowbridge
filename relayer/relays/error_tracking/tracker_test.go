package error_tracking

import (
	"errors"
	"testing"
)

func TestIsTransientError_WasmPanicIsPermanent(t *testing.T) {
	// Real-world stderr from a gas estimator wasm panic: contains the word
	// "unreachable" which would otherwise match the network transient pattern.
	err := errors.New(`gas estimator execution failed: exit status 1: ` +
		`Error: Invalid command: Failed to query delivery fees: ` +
		"wasm trap: wasm `unreachable` instruction executed\n" +
		`WASM backtrace: rust_begin_unwind ... panic_fmt ... XcmPaymentApi_query_delivery_fees`)
	if IsTransientError(err) {
		t.Fatal("wasm panic must classify as permanent, not transient")
	}
}

func TestIsTransientError_NetworkUnreachableIsTransient(t *testing.T) {
	err := errors.New("dial tcp 1.2.3.4:443: connect: network is unreachable")
	if !IsTransientError(err) {
		t.Fatal("network unreachable must classify as transient")
	}
}

func TestIsTransientError_NilIsNotTransient(t *testing.T) {
	if IsTransientError(nil) {
		t.Fatal("nil error should not classify as transient")
	}
}

func TestIsTransientError_UnknownIsPermanent(t *testing.T) {
	err := errors.New("some opaque error from a runtime call")
	if IsTransientError(err) {
		t.Fatal("unknown error should default to permanent")
	}
}

func TestIsTransientError_ClientIsClosedIsTransient(t *testing.T) {
	// Real-world: heartbeat Reconnect used to close the *ethclient.Client that
	// BoundContract still held; go-ethereum then returns "client is closed"
	// (not "websocket: close"). Keep classifying as transient as a backstop.
	err := errors.New("scan for sync tasks: check nonce relayed: check nonce from gateway contract: client is closed")
	if !IsTransientError(err) {
		t.Fatal("client is closed must classify as transient so fast-path retries after RPC failover")
	}
}

func TestIsTransientError_WaitReceiptTimeoutIsTransient(t *testing.T) {
	err := errors.New("watch multicall3 aggregate3 transaction: wait receipt timeout")
	if !IsTransientError(err) {
		t.Fatalf("expected wait receipt timeout to be transient, got permanent: %v", err)
	}
}

func TestIsTransientError_DNSFailureIsTransient(t *testing.T) {
	// Real-world stderr from the gas estimator hitting a DNS hiccup talking to
	// AssetHub. Previously fell through to permanent and dead-lettered a healthy
	// nonce.
	err := errors.New(`submit inbound message: gas estimation failed: ` +
		`gas estimator execution failed: exit status 1: Error: Connection error: ` +
		`Cannot connect to asset hub: RPC error: RPC error: client error: ` +
		`Failed to resolve IP addresses for this hostname: ` +
		`failed to lookup address information: Name or service not known`)
	if !IsTransientError(err) {
		t.Fatal("DNS resolution failure must classify as transient")
	}
}

func TestIsTransientError_IncompatibleCodegenIsTransient(t *testing.T) {
	// Real-world stderr after an Asset Hub runtime upgrade while ethereum-v2 still
	// runs a gas-estimator binary compiled against older subxt metadata. Previously
	// classified permanent and dead-lettered nonces (e.g. 665, 695, 696).
	err := errors.New(`submit inbound message: gas estimation failed: ` +
		`gas estimator execution failed: exit status 1: Error: Invalid command: ` +
		`Failed to dry run AssetHub XCM: Metadata(IncompatibleCodegen)`)
	if !IsTransientError(err) {
		t.Fatal("IncompatibleCodegen must classify as transient (stale estimator metadata)")
	}
	if !IsStaleMetadataError(err) {
		t.Fatal("IncompatibleCodegen must be detected as stale metadata for alerting")
	}
}

func TestIsTransientError_OnFinalityTooManyRequests(t *testing.T) {
	err := errors.New("check nonce from gateway contract: Too Many Requests, Please apply an OnFinality API key or additional credits to increase your limits. https://app.onfinality.io")
	if !IsTransientError(err) {
		t.Fatal("OnFinality Too Many Requests must stay transient so the process does not crash")
	}
}

func TestIsTransientError_HTTPStatus429(t *testing.T) {
	err := errors.New("suggest relay gas price: 429 Too Many Requests")
	if !IsTransientError(err) {
		t.Fatal("HTTP 429 must stay transient")
	}
}

func TestIsTransientError_StatusCode429(t *testing.T) {
	err := errors.New("eth_call: status code 429")
	if !IsTransientError(err) {
		t.Fatal("status code 429 must stay transient so Docker does not bounce the process")
	}
}

func TestIsTransientError_NonceContaining429IsNotTransient(t *testing.T) {
	err := errors.New("scan for sync tasks: nonce 429 already processed")
	if IsTransientError(err) {
		t.Fatal("an unrelated nonce number must not classify as transient")
	}
}

func TestIsTransientError_AlchemyFreeGetLogsRangeIsTransient(t *testing.T) {
	err := errors.New(`find events: filter events: 400 Bad Request: {"jsonrpc":"2.0","id":4,"error":{"code":-32600,"message":"Under the Free tier plan, you can make eth_getLogs requests with up to a 10 block range. Based on your parameters, this block range should work: [0x18b8c13, 0x18b8c1c]. Upgrade to PAYG for expanded block range."}}`)
	if !IsTransientError(err) {
		t.Fatal("Alchemy Free 10-block eth_getLogs 400 must be transient so the process does not crash")
	}
}

func TestIsTransientError_DrpcFreePlanRequestTimeoutIsTransient(t *testing.T) {
	err := errors.New(`find event GatewayInboundMessageDispatched: iter dispatch event: 408 Request Timeout: {"id":138,"jsonrpc":"2.0","error":{"message":"Request timeout on the free plan, please upgrade to paid plan","code":30}}`)
	if !IsTransientError(err) {
		t.Fatal("drpc free-plan 408 must be transient so the reward relayer does not crash")
	}
}

func TestIsStaleMetadataError_OtherErrorsDoNotMatch(t *testing.T) {
	if IsStaleMetadataError(nil) {
		t.Fatal("nil must not classify as stale metadata")
	}
	err := errors.New("dial tcp 1.2.3.4:443: connect: network is unreachable")
	if IsStaleMetadataError(err) {
		t.Fatal("ordinary transient errors must not trigger the stale-metadata alert")
	}
}
