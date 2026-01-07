#!/usr/bin/env bash

set -eux

forge script scripts/DeployWestendBeefyClient.sol:DeployWestendBeefyClient \
      --chain "${ETH_NETWORK}" \
      --rpc-url "${ETH_WS_ENDPOINT}" \
      --private-key "${PRIVATE_KEY}" \
      --etherscan-api-key "${ETHERSCAN_API_KEY}" \
      --verifier "etherscan" \
      --verify \
      --retries 10 \
      --broadcast \
      -vvvvv \
      
