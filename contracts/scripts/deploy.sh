#!/usr/bin/env bash

set -eux

forge script "scripts/Deploy.sol:${1}" \
    --chain-id 1 \
    --rpc-url "${MAINNET_RPC_URL}" \
    --ledger \
    --mnemonic-derivation-path "${MNEMONIC_DERIVATION_PATH}" \
    --broadcast \
    --verify \
    --verifier etherscan \
    --verifier-url "${ETHERSCAN_API_KEY}"
    --optimize \
    --via-ir \
    --optimizer-runs 100000 \
    -vvv
