#!/usr/bin/env bash

set -eux

export FOUNDRY_PROFILE=production

forge script \
      --rpc-url "${ETH_WS_ENDPOINT}" \
      --broadcast \
      --legacy \
      --with-gas-price 110000000000 \
      --verify \
      --etherscan-api-key "${ETHERSCAN_API_KEY}" \
      -vvvvv \
      scripts/DeployLocal.sol:DeployLocal
