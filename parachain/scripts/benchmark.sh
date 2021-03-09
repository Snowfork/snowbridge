#!/usr/bin/env bash

cargo build --release --features runtime-benchmarks

PALLETS="pallet_balances pallet_timestamp verifier_lightclient"

echo "Generating weights module with pallets ${PALLETS}"

rm runtime/src/weights/mod.rs

for pallet in $PALLETS
do
    # TODO: enable options in comments below once
    # all pallets work in wasm
    #    --execution wasm \
    #    --wasm-execution compiled \
    target/release/artemis benchmark \
        --chain spec.json \
        --pallet "${pallet}" \
        --extrinsic "*" \
        --repeat 20 \
        --steps 50 \
        --output runtime/src/weights/${pallet}_weights.rs
    echo "pub mod ${pallet}_weights;" >> runtime/src/weights/mod.rs
done
