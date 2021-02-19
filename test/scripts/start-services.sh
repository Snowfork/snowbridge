#!/usr/bin/env bash
set -eu

# config directory
configdir=$(mktemp -d -t artemis-config-XXX)

start_ganache()
{
    echo "Starting Ganache"
    NODE_OPTIONS="--max_old_space_size=8192" yarn run ganache-cli \
        --port=8545 \
        --blockTime=4 \
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

    truffle deploy --network e2e_test

    echo "Generating configuration from contracts"
    truffle exec scripts/dumpTestConfig.js $configdir --network e2e_test
    popd

    echo "Wrote configuration to $configdir"
}


start_parachain()
{
    echo "Starting Parachain"
    pushd ../parachain
    bin=$(pwd)/target/release/artemis

    cargo build --release --features "test-e2e"

    echo "Generating Parachain spec"
    target/release/artemis build-spec --disable-default-bootnode > $configdir/spec.json

    echo "Inserting Ganache chain info into genesis spec"
    ethereum_initial_header=$(curl http://localhost:8545 \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
        | node ../test/scripts/helpers/transformEthHeader.js)
    node ../test/scripts/helpers/overrideParachainSpec.js $configdir/spec.json \
        genesis.runtime.verifierLightclient.initialDifficulty 0x0 \
        genesis.runtime.verifierLightclient.initialHeader "$ethereum_initial_header" \
        genesis.runtime.parachainInfo.parachainId 200 \
        para_id 200

    echo "Writing Polkadot configuration"
    cp config.json $configdir/polkadotConfig.json
    parachain_conf="{
        \"bin\": \"$bin\",
        \"id\": \"200\",
        \"wsPort\": 11144,
        \"port\": 31200,
        \"flags\": [
            \"--execution=native\",
            \"-lruntime=debug,import_header=trace,bridge=trace\",
            \"--rpc-cors=all\",
            \"--offchain-worker=Always\",
            \"--enable-offchain-indexing=true\",
            \"--\",
            \"--execution=wasm\"
        ],
        \"chain\": \"$configdir/spec.json\"
    }"
    node ../test/scripts/helpers/overrideParachainSpec.js $configdir/polkadotConfig.json \
        parachains.0 "$parachain_conf"

    polkadot-launch $configdir/polkadotConfig.json &

    popd

    scripts/wait-for-it.sh -t 20 localhost:11144
    echo "Waiting for consensus between polkadot and parachain"
    sleep 60
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

trap 'kill $(jobs -p)' SIGINT SIGTERM EXIT

start_ganache
deploy_contracts
start_parachain
start_relayer

echo "Process Tree:"
pstree $$

sleep 3
until $(grep "Syncing headers starting..." $(pwd)/relay.log > /dev/null); do
    echo "Waiting for relayer to generate the DAG cache. This can take up to 20 minutes."
    sleep 20
done

until $(grep "Done retrieving finalized headers" $(pwd)/relay.log > /dev/null); do
    echo "Waiting for relayer to sync headers..."
    sleep 5
done

echo "System has been initialized"

wait
