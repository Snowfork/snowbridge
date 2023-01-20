#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/deploy-polkadot.sh
source scripts/configure-contracts.sh

start_chains()
{   
    #1 start polkadot relaychain and snowbridge parachain
    echo "Starting relaychain and snowbridge parachain"
    deploy_polkadot
    #2 initialize bridge contracts
    echo "Initializing bridge contracts"
    configure_contracts
    echo "Chains initialized ready"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start polkadot only!"
    trap kill_polkadot SIGINT SIGTERM EXIT
    check_tool && build_relaychain && build_parachain && start_chains
    wait
fi

