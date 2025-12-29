#!/bin/bash

include .env

# Makefile for deploying and testing cross-chain greeter contracts
deploy-l1-greeter:
	forge script DeployGreeter --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

deploy-l2-greeter:
	IS_L1=false forge script DeployGreeter --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

set-l1-remote:
	forge script SetRemote --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

set-l2-remote:
	IS_L1=false forge script SetRemote --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv

test-l1-greeting:
	forge script TestGreeting --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

test-l2-greeting:
	IS_L1=false forge script TestGreeting --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv

##############################################################
# Snowbridge Adaptors
deploy-snowbridge-l1-adaptor:
	forge script DeploySnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

deploy-snowbridge-l2-adaptor:
	forge script DeploySnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

test-snowbridge-l1-adaptor:
	forge script TestSnowbridgeL1Adaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv

test-snowbridge-l2-adaptor:
	forge script TestSnowbridgeL2Adaptor --chain $(BASE_CHAIN) --rpc-url $(BASE_RPC_URL) --broadcast -vvvv


##############################################################
# Uniswap Sepolia Swap

test-uniswap-sepolia-swap:
	forge script SwapScript --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --sender $(SEPOLIA_SENDER) --private-key $(DEPLOYER_KEY) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv
