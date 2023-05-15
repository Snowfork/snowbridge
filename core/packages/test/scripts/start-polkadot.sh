#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/deploy-polkadot.sh

start_chains()
{
    #1 start polkadot relaychain and snowbridge parachain
    echo "Starting Polkadot, BridgeHub and Statemint"
    deploy_polkadot
    echo "Polkadot started!"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start polkadot only!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && build_relaychain && build_cumulus_from_source && start_chains
    wait
fi

