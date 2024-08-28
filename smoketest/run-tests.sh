#!/bin/sh

set -xe

cargo test --no-run

tests=(
  register_token

  send_token
  send_token_to_penpal
  transfer_token

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
