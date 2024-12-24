#!/bin/sh

set -xe

cargo test --no-run

tests=(
  # Native ETH
  send_native_eth
  transfer_native_eth

  # ERC20 Tests
  register_token
  send_token
  send_token_to_penpal
  transfer_token

  # PNA Tests
  register_polkadot_token 
  transfer_polkadot_token
  send_polkadot_token

  # System Pallet Tests
  set_pricing_params
  set_token_transfer_fees
  create_agent
  create_channel
  transfer_native_from_agent
  upgrade_gateway
)

for test in ${tests[@]}; do 
  cargo test --test $test -- --nocapture
done
