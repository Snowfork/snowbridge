#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/deploy-ethereum.sh
source scripts/deploy-polkadot.sh
source scripts/configure-contracts.sh

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
    echo "Deploying contracts"
    deploy_contracts
    # 1.3 deploy relaychain&parachain with polkadot-launch
    echo "Starting snowbridge chains with polkadot-launch"
    start_polkadot_launch
    # 1.4 initialize bridge contracts
    echo "Configuring contracts"
    configure_contracts
    echo "Chains initialized ready"
}

if [ -z "${from_start_services:-}" ]; then
    echo "start chains only!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && cleanup && kill_all && build_parachain && start_chains
    wait
fi

