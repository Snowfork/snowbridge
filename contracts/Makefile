#!/bin/bash

include .env

l1-deploy-greeter:
	forge script DeployGreeter --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

l2-deploy-greeter:
	IS_L1=false forge script DeployGreeter --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l1-set-remote:
	forge script SetRemote --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

l2-set-remote:
	IS_L1=false forge script SetRemote --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv

l1-swap-and-greeting:
	forge script SendGreeting --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

l2-swap-and-greeting:
	IS_L1=false forge script SendGreeting --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv

deploy-snowbridge-l1-adaptor:
	forge script DeploySnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

deploy-snowbridge-l2-adaptor:
	forge script DeploySnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

test-snowbridge-l1-adaptor:
	forge script TestSnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

test-snowbridge-l2-adaptor:
	forge script TestSnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv

send-xcm-to-ethereum:
	forge script SendXcm --chain $(WESTEND_CHAIN_ID) --rpc-url $(WESTEND_ASSET_HUB_RPC_URL) --broadcast -vvvv
