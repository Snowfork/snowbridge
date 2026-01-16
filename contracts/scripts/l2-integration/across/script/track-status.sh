#!/bin/bash

set -eux

# Track the status of a deposit on Across
# Origin chain ID: Ethereum Mainnet (1)
# Deposit ID: 3507622 from SpokePool.FundsDeposited event,can build index from there

curl -H 'Content-Type: application/json' -s "https://app.across.to/api/deposit/status?originChainId=1&depositId=3507622" | jq .


