#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

wait_execution_node_ready() {
    local initial_block=""
    while [ -z "$initial_block" ] || [ "$initial_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]; do
        echo "Waiting for geth to get initial block..."
        initial_block=$(curl -H "Content-Type: application/json" -X POST --data '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x10", true],"id":1}' "$eth_endpoint_http" |
            jq -r '.result.stateRoot' || true)
        echo $initial_block
        sleep 3
    done
}

deploy_contracts() {
    wait_execution_node_ready
    pushd "$contract_dir"
    if [ "$eth_network" != "localhost" ]; then
        forge script \
            --rpc-url $eth_endpoint_http \
            --broadcast \
            --verify \
            --etherscan-api-key $etherscan_api_key \
            -vvv \
            src/DeployScript.sol:DeployScript
    else
        forge script \
            --rpc-url $eth_endpoint_http \
            --broadcast \
            -vvv \
            src/DeployScript.sol:DeployScript
    fi
    popd

    pushd "$test_helpers_dir"
    pnpm generateContracts "$output_dir/contracts.json"
    popd

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

if [ -z "${from_start_services:-}" ]; then
    echo "Deploying contracts"
    deploy_contracts
fi
