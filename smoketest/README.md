# Smoketests

Smoke tests for a running E2E environment

Currently it only supports coverage for the following:

From Ethereum:
* Registering new tokens on the AssetHub parachain
* Sending tokens to the AssetHub parachain

# Setup

1. First make sure the E2E Stack is running. See [web/packages/test/README.md](../web/packages/test/README.md).

2. Generate Rust bindings for both sides of the bridge

```shell
./make-bindings.sh
```

# Run Tests

Send an ethereum transaction to register a new token
```
cargo test --test register_token -- --nocapture
```

Send an ethereum transaction to send tokens
```
cargo test --test send_token -- --nocapture
```

Send an upgrade transaction via the relaychain.
Tests that the upgrade path works in terms of message routing and abi encoding.
This operation will brick the bridge as it upgrades the gateway to a mock gateway which has no implementation.
Please restart the testnet after running the test.
```
cargo test --test upgrade_gateway -- --nocapture
```

Send a substrate transaction to send tokens using bridge-transfer pallet on asset hub.
```
cargo test --test bridge_transfer_token -- --nocapture
```