#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

start_geth() {
    pushd "$root_dir/.."

    GETH_PATH="go-ethereum-lightclient/build/bin/geth"

    # Install Electra geth binary
    if [ ! -f "$GETH_PATH" ]; then
      echo "Local geth binary not found at $GETH_PATH."
      echo "Cloning and building go-ethereum-lightclient..."

      git clone https://github.com/lightclient/go-ethereum.git go-ethereum-lightclient
      pushd go-ethereum-lightclient
      git checkout prague-devnet-4
      make geth

      ./build/bin/geth version

      popd
    else
      echo "Local geth binary already exists at $GETH_PATH. Skipping clone and build."
    fi

    echo "Starting geth local node"
    ./go-ethereum-lightclient/build/bin/geth \
      --datadir "$output_dir/ethereum" \
      --state.scheme=hash \
      init "$config_dir/genesis-electra.json"
    ./go-ethereum-lightclient/build/bin/geth \
      --networkid 11155111 \
      --vmdebug \
      --datadir "$output_dir/ethereum" \
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
      --authrpc.jwtsecret "$config_dir/jwtsecret" \
      --password /dev/null \
      --rpc.gascap 0 \
      --ws.origins "*" \
      --gcmode archive \
      --syncmode=full \
      --state.scheme=hash \
      > "$output_dir/geth.log" 2>&1 &

      popd
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
