#!/usr/bin/env bash

set -eux

DEPLOY_RPC_URL="${DEPLOY_RPC_URL:-${ETH_WS_ENDPOINT}}"

forge script scripts/upgrade/DeployGateway.sol:DeployGateway \
      --chain "${ETH_NETWORK}" \
      --rpc-url "${DEPLOY_RPC_URL}" \
      --private-key "${PRIVATE_KEY}" \
      --etherscan-api-key "${ETHERSCAN_API_KEY}" \
      --verifier "etherscan" \
      --verify \
      --retries 20 \
      --broadcast \
      --slow \
      -vvvvv
