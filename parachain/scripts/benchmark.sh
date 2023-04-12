#!/usr/bin/env bash

# Example command for updating pallet benchmarking
pushd ../cumulus
cargo run --release --bin polkadot-parachain \
--features runtime-benchmarks \
-- \
benchmark pallet \
--chain=bridge-hub-rococo-dev \
--pallet=snowbridge_ethereum_beacon_client \
--extrinsic="*" \
--execution=wasm --wasm-execution=compiled \
--steps 50 --repeat 20 \
--output ../parachain/pallets/ethereum-beacon-client/src/mainnet_weights.rs
popd
