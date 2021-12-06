#!/usr/bin/env bash

# Example command for updating pallet benchmarking
target/release/snowbridge benchmark \
  --chain spec.json \
  --execution wasm \
  --wasm-execution compiled \
  --pallet 'incentivized_channel::inbound' \
  --extrinsic '*' \
  --repeat 20 \
  --steps 50 \
  --output pallets/incentivized-channel/src/inbound/weights.rs \
  --template module-weight-template.hbs
