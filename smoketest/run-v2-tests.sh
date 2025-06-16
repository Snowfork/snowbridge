#!/bin/sh

set -xe

cargo test --no-run

tests=(
  # ERC20 Tests
  register_ena
  send_ena_to_ah
  send_ena_to_penpal
  transfer_ena

  # PNA Tests
  register_pna
  transfer_pna
  send_pna

  # Transact
  transact_e2p
  transact_p2e
)

for test in ${tests[@]}; do 
  cargo test --test v2 $test -- --nocapture
done
