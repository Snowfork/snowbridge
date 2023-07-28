# Smoketests

Smoke tests for a running E2E environment

Currently it only supports coverage for the following:

From Ethereum:
* Registering new tokens on the AssetHub parachain
* Sending tokens to the AssetHub parachain

# Setup

1. First make sure the E2E Stack is running. See [core/packages/test/README.md](../core/packages/test/README.md).

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
