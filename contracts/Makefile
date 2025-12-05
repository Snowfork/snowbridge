#!/bin/bash

include .env

dc:
	forge script DeployComposer --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL)  --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv 

dr:
	forge script DeployReceiver --chain $(OPT_CHAIN) --rpc-url $(OPT_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 10 --broadcast -vvvv

da:
	forge script DeployAdaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 20 --broadcast -vvvv

et:
	forge script TransferEth --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv 

eta:
	forge script TransferEthFromAdaptor --chain $(SEPOLIA_CHAIN) --rpc-url $(SEPOLIA_RPC_URL) --broadcast -vvvv 
