#!/bin/bash

##############################################################
# Snowbridge Adaptors
deploy-snowbridge-l1-adaptor:
	forge script DeploySnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --private-key ${PRIVATE_KEY} --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 20 --broadcast -vvvv 

deploy-snowbridge-l2-adaptor:
	forge script DeploySnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --private-key ${PRIVATE_KEY} --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 20 --broadcast -vvvv

test-snowbridge-l1-adaptor:
	forge script TestSnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l1-adaptor-native-ether:
	forge script TestSnowbridgeL1AdaptorNativeEther --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l2-adaptor:
	forge script TestSnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l2-adaptor-native-ether:
	forge script TestSnowbridgeL2AdaptorNativeEther --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

##############################################################
# Uniswap Sepolia Swap

test-uniswap-sepolia-swap:
	forge script SwapScript --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --sender $(SEPOLIA_SENDER) --private-key $(DEPLOYER_KEY) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv
