#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/deploy-ethereum.sh

start_chains()
{   #1 start ethereum and deploy contracts
    echo "start ethereum and deploy bridge contrancts"
    deploy_ethereum
    echo "Ethereum started!"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start ethereum only!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && rm -rf "$ethereum_data_dir" && start_chains
    wait
fi
