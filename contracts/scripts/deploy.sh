#!/usr/bin/env bash

set -eux

forge script "scripts/DeployBeefyClient.sol:DeployBeefyClient" \
    --chain-id 1 \
    --rpc-url "${MAINNET_RPC_URL}" \
    --ledger \
    --mnemonic-derivation-paths "m/44'/60'/1'/0/0" \
    --broadcast \
    --verify \
    --optimize \
    --via-ir \
    --optimizer-runs 100000 \
    -vvvv
