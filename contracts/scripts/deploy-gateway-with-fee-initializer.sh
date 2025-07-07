#!/usr/bin/env bash

set -eux

export FOUNDRY_PROFILE=production

forge script \
      --rpc-url "${ETH_WS_ENDPOINT}" \
      --broadcast \
      --legacy \
      --with-gas-price 10000000000 \
      --verify \
      --delay 6 \
      --chain "${ETH_NETWORK}" \
      --etherscan-api-key "${ETHERSCAN_API_KEY}" \
      -vvvvv \
      scripts/DeployGatewayWithFeeInitializer.sol:DeployGatewayWithFeeInitializer
