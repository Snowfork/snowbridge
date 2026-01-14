#!/bin/bash

##############################################################
# Snowbridge Adaptors
deploy-snowbridge-l1-adaptor:
	forge script DeploySnowbridgeL1Adaptor --chain $(L1_NETWORK) --rpc-url $(L1_RPC_URL) --private-key ${PRIVATE_KEY} --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 20 --broadcast -vvvv 

deploy-snowbridge-l2-adaptor:
	forge script DeploySnowbridgeL2Adaptor --chain $(L2_NETWORK) --rpc-url $(L2_RPC_URL) --private-key ${PRIVATE_KEY} --etherscan-api-key $(ETHERSCAN_API_KEY) --verifier $(VERIFIER) --verify --retries 20 --broadcast -vvvv

test-snowbridge-l1-adaptor:
	forge script TestSnowbridgeL1Adaptor --chain $(L1_NETWORK) --rpc-url $(L1_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l1-adaptor-native-ether:
	forge script TestSnowbridgeL1AdaptorNativeEther --chain $(L1_NETWORK) --rpc-url $(L1_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l2-adaptor:
	forge script TestSnowbridgeL2Adaptor --chain $(L2_NETWORK) --rpc-url $(L2_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv

test-snowbridge-l2-adaptor-native-ether:
	forge script TestSnowbridgeL2AdaptorNativeEther --chain $(L2_NETWORK) --rpc-url $(L2_RPC_URL) --private-key ${PRIVATE_KEY} --broadcast -vvvv
