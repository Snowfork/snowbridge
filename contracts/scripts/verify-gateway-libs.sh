#!/usr/bin/env bash

set -eux

# Verify linked libraries for GatewaySepolia202603 on Sepolia
# These must be verified separately after the main gateway contract is deployed

ETHERSCAN_API_KEY="${ETHERSCAN_API_KEY}"
CHAIN="${ETH_NETWORK:-sepolia}"

# Library addresses from Sepolia deployment of GatewaySepolia202603
VERIFICATION_LIB="0x23d6c0bb025570b291b834cd48b78984cffc6e5c"
CALLSV1_LIB="0x3ef6c570c43761cdcd3b05fbef120ecfddbf1104"

echo "Verifying linked libraries on $CHAIN..."

# Verify Verification library
echo "Verifying Verification library at $VERIFICATION_LIB..."
forge verify-contract "$VERIFICATION_LIB" \
  src/Verification.sol:Verification \
  --chain "$CHAIN" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch

# Verify CallsV1 library
echo "Verifying CallsV1 library at $CALLSV1_LIB..."
forge verify-contract "$CALLSV1_LIB" \
  src/v1/Calls.sol:CallsV1 \
  --chain "$CHAIN" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch

echo "Library verification complete!"
