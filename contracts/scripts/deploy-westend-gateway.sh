#!/usr/bin/env bash

set -eux

forge script scripts/upgrade/DeployGateway202601.sepolia.sol:DeployGateway202601 \
      --chain "${ETH_NETWORK}" \
      --rpc-url "${ETH_WS_ENDPOINT}" \
      --private-key "${PRIVATE_KEY}" \
      --etherscan-api-key "${ETHERSCAN_API_KEY}" \
      --verifier "etherscan" \
      --verify \
      --retries 20 \
      --broadcast \
      -vvvvv
