#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/deploy-ethereum.sh
source scripts/deploy-polkadot.sh
source scripts/configure-contracts.sh

start_chains()
{   #1 start ethereum and deploy contracts
    echo "start ethereum and deploy bridge contrancts"
    deploy_ethereum
    #2 start polkadot relaychain and snowbridge parachain
    echo "Starting relaychain and snowbridge parachain"
    deploy_polkadot
    #3 initialize bridge contracts
    echo "Initializing bridge contracts"
    configure_contracts
    echo "Chains initialized ready"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start chains only!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && cleanup && kill_all && build_parachain && start_chains
    wait
fi

