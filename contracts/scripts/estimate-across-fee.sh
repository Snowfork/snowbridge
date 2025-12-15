# !/bin/bash

set -eux

# Estimate fees for transferring 1 USDC from Sepolia to Base via Across
# Input token: Sepolia USDC (0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238)
# Output token: Base USDC (0x036CbD53842c5426634e7929541eC2318f3dCF7e)
# Origin chain ID: Sepolia (11155111)
# Destination chain ID: Base (84532)
# Amount: 1 USDC (1,000,000 in 6 decimal places)
curl -L \
  'https://testnet.across.to/api/suggested-fees?inputToken=0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238&outputToken=0x036CbD53842c5426634e7929541eC2318f3dCF7e&originChainId=11155111&destinationChainId=84532&amount=1000000' | jq .

# Estimate fees for transferring 0.001 WETH from Sepolia to Base via Across
# Input token: Sepolia WETH (0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14)
# Output token: Base WETH (0x4200000000000000000000000000000000000006)
# Origin chain ID: Sepolia (11155111)
# Destination chain ID: Base (84532)
# Amount: 0.001 WETH (1,000,000,000,000,000 in 18 decimal places)
curl -L \
  'https://testnet.across.to/api/suggested-fees?inputToken=0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14&outputToken=0x4200000000000000000000000000000000000006&originChainId=11155111&destinationChainId=84532&amount=1000000000000000' | jq .

# Estimate swap approval data for transferring 1 USDC from Sepolia to Base via Across
# Input token: Sepolia USDC (0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238)
# Output token: Base USDC (0x036CbD53842c5426634e7929541eC2318f3dCF7e)
# Origin chain ID: Sepolia (11155111)
# Destination chain ID: Base (84532)
# Amount: 1 USDC (1,000,000 in 6 decimal places)
# Depositor: 0xA4d353BBc130cbeF1811f27ac70989F9d568CeAB
curl -L \
  'https://testnet.across.to/api/swap/approval?tradeType=minOutput&inputToken=0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238&outputToken=0x036CbD53842c5426634e7929541eC2318f3dCF7e&originChainId=11155111&destinationChainId=84532&amount=1000000&&depositor=0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd' | jq .
