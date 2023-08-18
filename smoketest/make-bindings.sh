#!/usr/bin/env bash

set -eu

mkdir -p src/contracts

# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGateway|WETH9|GatewayUpgradeMock|HelloWorld|AgentExecutor' \
    --bindings-path src/contracts \
    --root ../contracts

# Generate Rust bindings for BridgeHub
subxt_version=v0.27.1
cargo_dir=".cargo"
export PATH=$PATH:$cargo_dir/bin

# Install subxt
command -v subxt || cargo install subxt-cli \
    --git https://github.com/paritytech/subxt.git \
    --tag $subxt_version \
    --root $cargo_dir

if ! lsof -Pi :11144 -sTCP:LISTEN -t >/dev/null; then
    echo "substrate nodes not running, please start with the e2e setup and rerun this script"
    exit 1
fi

# Fetch metadata from BridgeHub and generate client
subxt codegen --url ws://localhost:11144 | rustfmt +nightly-"$SNOWBRIDGE_RUST_NIGHTLY" --edition 2021 --emit=stdout >src/parachains/bridgehub.rs
subxt codegen --url ws://localhost:12144 | rustfmt +nightly-"$SNOWBRIDGE_RUST_NIGHTLY" --edition 2021 --emit=stdout >src/parachains/assethub.rs
subxt codegen --url ws://localhost:9944 | rustfmt +nightly-"$SNOWBRIDGE_RUST_NIGHTLY" --edition 2021 --emit=stdout >src/parachains/relaychain.rs
subxt codegen --url ws://localhost:13144 | rustfmt +nightly-"$SNOWBRIDGE_RUST_NIGHTLY" --edition 2021 --emit=stdout >src/parachains/template.rs
