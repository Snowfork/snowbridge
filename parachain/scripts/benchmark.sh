#!/usr/bin/env bash

# Example command for updating pallet benchmarking
target/release/snowbridge benchmark \
  --chain spec.json \
  --execution wasm \
  --wasm-execution compiled \
  --pallet 'basic_channel::inbound' \
  --extrinsic '*' \
  --repeat 20 \
  --steps 50 \
  --output pallets/basic-channel/src/inbound/weights.rs \
  --template templates/module-weight-template.hbs
