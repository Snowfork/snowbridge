#!/usr/bin/env bash

set -eu

mkdir -p src/contracts

subxt_version=v0.27.1
cargo_dir=".cargo"
export PATH=$PATH:$cargo_dir/bin

command -v subxt || cargo install subxt-cli \
    --git https://github.com/paritytech/subxt.git \
    --tag $subxt_version \
    --root $cargo_dir

subxt codegen --url ws://localhost:11144 | rustfmt --edition 2021 --emit=stdout > src/parachains/bridgehub.rs
subxt codegen --url ws://localhost:12144 | rustfmt --edition 2021 --emit=stdout > src/parachains/statemine.rs
forge bind --module --overwrite --bindings-path src/contracts --root ../core/packages/contracts
