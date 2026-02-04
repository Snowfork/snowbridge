#!/usr/bin/env bash

set -eu

# Use underscore names so the functions are valid in bash.
test_snowbridge_l1_adaptor() {
  forge script TestSnowbridgeL1Adaptor --chain "${L1_NETWORK}" --rpc-url "${L1_RPC_URL}" --private-key "${PRIVATE_KEY}" --slow --broadcast -vvvv
}

test_snowbridge_l1_adaptor_native_ether() {
  forge script TestSnowbridgeL1AdaptorNativeEther --chain "${L1_NETWORK}" --rpc-url "${L1_RPC_URL}" --private-key "${PRIVATE_KEY}" --slow --broadcast -vvvv
}

test_snowbridge_l2_adaptor() {
  forge script TestSnowbridgeL2Adaptor --chain "${L2_NETWORK}" --rpc-url "${L2_RPC_URL}" --private-key "${PRIVATE_KEY}" --broadcast -vvvv
}

test_snowbridge_l2_adaptor_native_ether() {
  forge script TestSnowbridgeL2AdaptorNativeEther --chain "${L2_NETWORK}" --rpc-url "${L2_RPC_URL}" --private-key "${PRIVATE_KEY}" --broadcast -vvvv
}

test_snowbridge_l2_adaptor_weth() {
  forge script TestSnowbridgeL2AdaptorWeth --chain "${L2_NETWORK}" --rpc-url "${L2_RPC_URL}" --private-key "${PRIVATE_KEY}" --broadcast -vvvv
}

test_uniswap_quoter() {
  forge script TestUniswapQuoter --chain "${L1_NETWORK}" --rpc-url "${L1_RPC_URL}" --private-key "${PRIVATE_KEY}" --broadcast -vvvv
}

usage() {
  cat <<'EOF'
Usage: ./scripts/test-l2-adaptors.sh <target>

Targets:
  l1              Run TestSnowbridgeL1Adaptor
  l1-native       Run TestSnowbridgeL1AdaptorNativeEther
  l2              Run TestSnowbridgeL2Adaptor
  l2-native       Run TestSnowbridgeL2AdaptorNativeEther
  l2-weth         Run TestSnowbridgeL2AdaptorWeth
  l1-swap-quoter  Run TestUniswapQuoter
  all             Run all of the above in sequence
EOF
}

if [[ ${1-} == "" ]]; then
  usage
  exit 1
fi

case "$1" in
  l1)          test_snowbridge_l1_adaptor ;;
  l1-native)   test_snowbridge_l1_adaptor_native_ether ;;
  l2)          test_snowbridge_l2_adaptor ;;
  l2-native)   test_snowbridge_l2_adaptor_native_ether ;;
  l2-weth)     test_snowbridge_l2_adaptor_weth ;;
  l1-swap-quoter) test_uniswap_quoter ;;
  all)
    test_snowbridge_l1_adaptor
    test_snowbridge_l1_adaptor_native_ether
    test_snowbridge_l2_adaptor
    test_snowbridge_l2_adaptor_native_ether
    test_snowbridge_l2_adaptor_weth
    ;;
  *)
    usage
    exit 1
    ;;
esac


