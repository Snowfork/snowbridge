#!/usr/bin/env bash

set -eu

deploy_snowbridge_l1_adaptor() {
  forge script DeploySnowbridgeL1Adaptor --chain "${L1_NETWORK}" --rpc-url "${L1_RPC_URL}" --private-key "${PRIVATE_KEY}" --etherscan-api-key "${ETHERSCAN_API_KEY}" --verifier "${VERIFIER}" --verify --retries 20 --broadcast -vvvv
}

deploy_snowbridge_l2_adaptor() {
  forge script DeploySnowbridgeL2Adaptor --chain "${L2_NETWORK}" --rpc-url "${L2_RPC_URL}" --private-key "${PRIVATE_KEY}" --etherscan-api-key "${ETHERSCAN_API_KEY}" --verifier "${VERIFIER}" --verify --retries 20 --broadcast -vvvv
}

usage() {
  cat <<'EOF'
Usage: ./scripts/deploy-l2-adaptors.sh <target>

Targets:
  l1   Deploy Snowbridge L1 adaptor
  l2   Deploy Snowbridge L2 adaptor
  all  Deploy both
EOF
}

if [[ ${1-} == "" ]]; then
  usage
  exit 1
fi

case "$1" in
  l1)  deploy_snowbridge_l1_adaptor ;;
  l2)  deploy_snowbridge_l2_adaptor ;;
  all)
    deploy_snowbridge_l1_adaptor
    deploy_snowbridge_l2_adaptor
    ;;
  *)
    usage
    exit 1
    ;;
esac


