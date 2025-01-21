#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
export output_electra_dir="/tmp/electra"

start_geth() {
    rm -rf $output_electra_dir
    mkdir -p $output_electra_dir
    mkdir -p $output_electra_dir/ethereum
    cp config/genesis-electra.json $output_electra_dir
    cp config/jwtsecret $output_electra_dir

    echo "Starting geth local node"
    docker run --rm \
      -v "${output_electra_dir}:/mnt" \
      docker.io/ethpandaops/geth:lightclient-prague-devnet-4 \
      --datadir /mnt/ethereum \
      --state.scheme=hash \
      init /mnt/genesis-electra.json
    docker run --rm -m=12g --memory-reservation=8g --cpus 2 \
      -v "${output_electra_dir}:/mnt" \
      -p 8551:8551 \
      -p 8545:8545 \
      -p 8546:8546 \
      --env 'NODE_OPTIONS=--max-old-space-size=8192' \
      docker.io/ethpandaops/geth:lightclient-prague-devnet-4 \
      --networkid 11155111 \
      --vmdebug \
      --datadir /mnt/ethereum \
      --http \
      --http.api debug,personal,eth,net,web3,txpool,engine \
      --ws --ws.api debug,eth,net,web3 \
      --rpc.allow-unprotected-txs \
      --authrpc.addr 0.0.0.0 \
      --authrpc.vhosts "*" \
      --http \
      --http.api "debug,personal,eth,net,web3,txpool,engine,miner" \
      --http.addr 0.0.0.0 \
      --http.vhosts "*" \
      --http.corsdomain '*' \
      --ws \
      --ws.api "debug,eth,net,web3" \
      --ws.addr 0.0.0.0 \
      --ws.origins "*" \
      --allow-insecure-unlock \
      --authrpc.jwtsecret mnt/jwtsecret \
      --password /dev/null \
      --rpc.gascap 0 \
      --ws.origins "*" \
      --gcmode archive \
      --syncmode=full \
      --state.scheme=hash \
      > "$output_electra_dir/geth.log" 2>&1 &
}

start_lodestar() {
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
    ./lodestar --version
    ./lodestar dev \
        --genesisValidators 8 \
        --genesisTime $timestamp \
        --startValidators "0..7" \
        --enr.ip6 "127.0.0.1" \
        --rest.address "0.0.0.0" \
        --eth1.providerUrls "http://$HOST:8545" \
        --execution.urls "http://$HOST:8551" \
        --dataDir "$ethereum_data_dir" \
        --reset \
        --terminal-total-difficulty-override 0 \
        --genesisEth1Hash $genesisHash \
        --params.ALTAIR_FORK_EPOCH 0 \
        --params.BELLATRIX_FORK_EPOCH 0 \
        --params.CAPELLA_FORK_EPOCH 0 \
        --params.DENEB_FORK_EPOCH 0 \
        --params.ELECTRA_FORK_EPOCH 0 \
        --eth1=true \
        --rest.namespace="*" \
        --jwt-secret $config_dir/jwtsecret \
        --chain.archiveStateEpochFrequency 1 \
        >"$output_dir/lodestar.log" 2>&1 &
    popd
}

deploy_local() {
    # 1. deploy execution client
    echo "Starting execution node"
    start_geth

    echo "Waiting for geth API to be ready"
    sleep 10

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
