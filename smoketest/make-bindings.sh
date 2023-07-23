#!/usr/bin/env bash

set -eu

mkdir -p src/contracts

# Generate Rust bindings for BridgeHub
subxt_version=v0.27.1
cargo_dir=".cargo"
export PATH=$PATH:$cargo_dir/bin

# Install subxt
command -v subxt || cargo install subxt-cli \
    --git https://github.com/paritytech/subxt.git \
    --tag $subxt_version \
    --root $cargo_dir

# Fetch metadata from BridgeHub and generate client
subxt codegen --url ws://localhost:11144 | rustfmt --edition 2021 --emit=stdout > src/parachains/bridgehub.rs

# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --bindings-path src/contracts \
    --select 
    --root ../core/packages/contracts
