#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

start_geth() {
    if [ "$eth_network" == "localhost" ]; then
        echo "Starting geth local node"
        cp config/genesis.json "$output_dir/genesis.json"
        geth init --datadir "$ethereum_data_dir" "$output_dir/genesis.json"
        geth account import --datadir "$ethereum_data_dir" --password /dev/null config/dev-example-key0.prv
        geth account import --datadir "$ethereum_data_dir" --password /dev/null config/dev-example-key1.prv
        geth --vmdebug --datadir "$ethereum_data_dir" --networkid 15 \
            --http --http.api debug,personal,eth,net,web3,txpool,engine,miner --ws --ws.api debug,eth,net,web3 \
            --rpc.allow-unprotected-txs --mine --miner.threads=1 \
            --miner.etherbase=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
            --authrpc.addr="127.0.0.1" \
            --http.addr="127.0.0.1" \
            --allow-insecure-unlock \
            --authrpc.jwtsecret config/jwtsecret \
            --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
            --password /dev/null \
            --rpc.gascap 0 \
            --ws.origins "*" \
            --trace "$ethereum_data_dir/trace" \
            --gcmode archive \
            --syncmode=full \
            > "$output_dir/geth.log" 2>&1 &
    fi
}

start_lodestar() {
    if [ "$eth_network" == "localhost" ]; then
        echo "Starting lodestar local node"
        genesisHash=$(curl $eth_endpoint_http \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')

        timestamp=$(date -d'+10second' +%s)

        npx lodestar dev \
            --genesisValidators 8 \
            --genesisTime $timestamp \
            --startValidators "0..7" \
            --enr.ip6 "127.0.0.1" \
            --eth1.providerUrls "http://127.0.0.1:8545" \
            --execution.urls "http://127.0.0.1:8551" \
            --dataDir "$output_dir/beacon-$timestamp" \
            --reset \
            --terminal-total-difficulty-override 0 \
            --genesisEth1Hash $genesisHash \
            --params.ALTAIR_FORK_EPOCH 0 \
            --params.BELLATRIX_FORK_EPOCH 0 \
            --eth1=true \
            --rest.namespace="*" \
            --jwt-secret config/jwtsecret \
            > "$output_dir/lodestar.log" 2>&1 &
    fi
}

deploy_contracts()
{
    pushd "$contract_dir"
    forge script \
        --rpc-url $eth_endpoint_http \
        --broadcast \
        -vvv \
        scripts/Deploy.sol:DeployScript
    node scripts/generateContractInfo.js "$output_dir/contracts.json"
    popd
    echo "Exported contract artifacts: $output_dir/contracts.json"
}

deploy_ethereum()
{
    # 1. deploy execution client
    echo "Starting execution node"
    start_geth
    echo "Waiting for geth API to be ready"
    sleep 3

    # 2. deploy consensus client
    echo "Starting beacon node"
    start_lodestar

    # 3. deploy bridge contracts
    echo "Deploying contracts"
    deploy_contracts
}
