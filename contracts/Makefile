#!/bin/bash

include .env

l1-deploy-greeter:
	forge script DeployGreeter --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

l2-deploy-greeter:
	IS_L1=false forge script DeployGreeter --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l1-set-remote:
	forge script SetRemote --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l2-set-remote:
	IS_L1=false forge script SetRemote --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l1-swap-and-greeting:
	forge script SendGreeting --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l2-swap-and-greeting:
	IS_L1=false forge script SendGreeting --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l2-deploy-snowbridge-frontend:
	forge script DeploySnowbridgeFrontend --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

l2-send-token-from-snowbridge-frontend:
	forge script SendTokenBySnowbridgeFrontend --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv
