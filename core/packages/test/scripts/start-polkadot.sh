#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/deploy-polkadot.sh

start_chains()
{
    #1 start polkadot relaychain and snowbridge parachain
    echo "Starting relaychain and snowbridge parachain"
    deploy_polkadot
    echo "Polkadot started!"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start polkadot only!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && build_relaychain_from_source && build_parachain && start_chains
    wait
fi

