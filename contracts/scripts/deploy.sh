#!/usr/bin/env bash

set -eux

export FOUNDRY_PROFILE=production

forge script "$1" \
    --chain-id 1 \
    --rpc-url "${MAINNET_RPC_URL}" \
    --ledger \
    --mnemonic-indexes "${MNEMONIC_INDEX}" \
    --sender "${SENDER_ADDRESS}" \
    --broadcast \
    --verify \
    -vvv
