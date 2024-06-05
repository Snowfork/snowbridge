#!/usr/bin/env bash

set -eux

export FOUNDRY_PROFILE=production

forge script "scripts/UpgradeShell.sol:UpgradeShell" \
    --chain-id 1 \
    --rpc-url "${MAINNET_RPC_URL}" \
    --ledger \
    --mnemonic-derivation-paths "${MNEMONIC_DERIVATION_PATH}" \
    -vvvv
