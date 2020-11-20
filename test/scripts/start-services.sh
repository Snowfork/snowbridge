#!/usr/bin/env bash
set -eu

# config directory
configdir=$(mktemp -d -t artemis-config-XXX)

start_ganache()
{
    echo "Starting Ganache"
    yarn run ganache-cli \
        --port=8545 \
        --blockTime=0 \
        --networkId=344 \
        --deterministic \
        --mnemonic='stone speak what ritual switch pigeon weird dutch burst shaft nature shove' \
        >ganache.log 2>&1 &

    scripts/wait-for-it.sh -t 20 localhost:8545
    sleep 5
}

deploy_contracts()
{
    echo "Deploying contracts"
    pushd ../ethereum

    truffle deploy --network ganache

    echo "Generating configuration from contracts"
    truffle exec scripts/dumpTestConfig.js $configdir --network ganache
    popd

    cp $configdir/test-config.json test-config.json

    echo "Wrote configuration to $configdir"
}


start_parachain()
{
    echo "Starting Parachain"
    logfile=$(pwd)/parachain.log
    pushd ../parachain

    source $configdir/parachain.env
    export ETH_APP_ID ERC20_APP_ID

    cargo build
    target/debug/artemis-node --tmp --dev >$logfile 2>&1 &

    popd

    scripts/wait-for-it.sh -t 20 localhost:9944
    sleep 5

    echo "Parachain PID: $!"
}

start_relayer()
{
    echo "Starting Relay"
    logfile=$(pwd)/relay.log
    pushd ../relayer

    mage build

    export ARTEMIS_ETHEREUM_KEY="0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
    export ARTEMIS_SUBSTRATE_KEY="//Relay"

    build/artemis-relay run --config $configdir/config.toml >$logfile 2>&1 &

    popd
    echo "Relay PID: $!"
}

trap 'kill $(jobs -p)' SIGINT SIGTERM

start_ganache
deploy_contracts
start_parachain
start_relayer

# TODO: Exit when any child process dies
#  https://stackoverflow.com/questions/37496896/exit-a-bash-script-when-one-of-the-subprocesses-exits

pstree $$

wait
