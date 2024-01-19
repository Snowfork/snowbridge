#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

start_geth() {
    if [ "$eth_network" == "localhost" ]; then
        echo "Starting geth local node"
        local timestamp="0" #start Cancun from genesis
        jq \
            --argjson timestamp "$timestamp" \
            '
            .config.CancunTime = $timestamp
            ' \
            config/genesis.json >$output_dir/genesis.json
        geth init --datadir "$ethereum_data_dir" "$output_dir/genesis.json"
        geth account import --datadir "$ethereum_data_dir" --password /dev/null config/dev-example-key0.prv
        geth account import --datadir "$ethereum_data_dir" --password /dev/null config/dev-example-key1.prv
        geth --vmdebug --datadir "$ethereum_data_dir" --networkid 11155111 \
            --http --http.api debug,personal,eth,net,web3,txpool,engine,miner --ws --ws.api debug,eth,net,web3 \
            --rpc.allow-unprotected-txs --mine \
            --miner.etherbase=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
            --authrpc.addr="127.0.0.1" \
            --http.addr="0.0.0.0" \
            --http.corsdomain '*' \
            --allow-insecure-unlock \
            --authrpc.jwtsecret config/jwtsecret \
            --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
            --password /dev/null \
            --rpc.gascap 0 \
            --ws.origins "*" \
            --trace "$ethereum_data_dir/trace" \
            --gcmode archive \
            --syncmode=full \
            >"$output_dir/geth.log" 2>&1 &
    fi
}

start_lodestar() {
    if [ "$eth_network" == "localhost" ]; then
        echo "Starting lodestar local node"
        local genesisHash=$(curl $eth_endpoint_http \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')
        echo "genesisHash is: $genesisHash"
        # use gdate here for raw macos without nix
        local timestamp=""
        if [[ "$(uname)" == "Darwin" && -z "${IN_NIX_SHELL:-}" ]]; then
            timestamp=$(gdate -d'+10second' +%s)
        else
            timestamp=$(date -d'+10second' +%s)
        fi

        export LODESTAR_PRESET="mainnet"

        pushd $root_dir/lodestar
        ./lodestar dev \
            --genesisValidators 8 \
            --genesisTime $timestamp \
            --startValidators "0..7" \
            --enr.ip6 "127.0.0.1" \
            --eth1.providerUrls "http://127.0.0.1:8545" \
            --execution.urls "http://127.0.0.1:8551" \
            --dataDir "$ethereum_data_dir" \
            --reset \
            --terminal-total-difficulty-override 0 \
            --genesisEth1Hash $genesisHash \
            --params.ALTAIR_FORK_EPOCH 0 \
            --params.BELLATRIX_FORK_EPOCH 0 \
            --params.CAPELLA_FORK_EPOCH 0 \
            --params.DENEB_FORK_EPOCH 0 \
            --eth1=true \
            --rest.namespace="*" \
            --jwt-secret $config_dir/jwtsecret \
            --chain.archiveStateEpochFrequency 1 \
            >"$output_dir/lodestar.log" 2>&1 &
        popd
    fi
}

set_slot_time() {
    local new_value=$1
    echo "Hack lodestar for faster slot time"
    local preset_mainnet_config_file="$web_dir/node_modules/.pnpm/@lodestar+config@$lodestar_version/node_modules/@lodestar/config/lib/chainConfig/presets/mainnet.js"
    if [[ "$(uname)" == "Darwin" && -z "${IN_NIX_SHELL:-}" ]]; then
        gsed -i "s/SECONDS_PER_SLOT: .*/SECONDS_PER_SLOT: $new_value,/g" $preset_mainnet_config_file
    else
        sed -i "s/SECONDS_PER_SLOT: .*/SECONDS_PER_SLOT: $new_value,/g" $preset_mainnet_config_file
    fi
}

deploy_local() {
    # 1. deploy execution client
    echo "Starting execution node"
    start_geth

    echo "Waiting for geth API to be ready"
    sleep 3

    # if were are running locally speed up the seconds per slot from 12 seconds to 2 seconds. if we are not
    # running locally, revert
    if [ "$eth_network" == "localhost" ]; then
        set_slot_time 1
    else
        set_slot_time 12
    fi

    # 2. deploy consensus client
    echo "Starting beacon node"
    start_lodestar
}

deploy_ethereum() {
    check_tool && rm -rf "$ethereum_data_dir" && deploy_local
}

if [ -z "${from_start_services:-}" ]; then
    echo "start ethereum only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_ethereum
    echo "ethereum local nodes started!"
    wait
fi
