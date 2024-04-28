#!/usr/bin/env bash

set -eux

forge script "scripts/Deploy.sol:${1}" \
    --chain-id 1 \
    --rpc-url "${MAINNET_RPC_URL}" \
    --ledger \
    --mnemonic-derivation-paths "${MNEMONIC_DERIVATION_PATH}" \
    --broadcast \
    --verify \
    --optimize \
    --via-ir \
    --optimizer-runs 100000 \
    -vvvv
