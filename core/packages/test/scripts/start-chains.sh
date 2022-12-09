#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/deploy-ethereum.sh
source scripts/deploy-polkadot.sh

start_chains()
{
    # 1.1 deploy execution&consensus client
    echo "Starting execution node"
    start_geth
    echo "Waiting for geth API to be ready"
    sleep 3
    echo "Starting beacon node"
    start_lodestar
    # 1.2 deploy bridge contracts
    deploy_contracts
    # 1.3 deploy relaychain&parachain with polkadot-launch
    start_polkadot_launch
}

trap forcekill SIGINT SIGTERM EXIT
check_build_tool && check_binary && cleanup
start_chains
wait
