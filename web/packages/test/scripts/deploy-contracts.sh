#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

deploy_command() {
    local deploy_script=$1

    pushd "$contract_dir"
    if [ "$eth_network" != "localhost" ]; then
        forge script \
            --rpc-url $eth_endpoint_http \
            --broadcast \
            --verify \
            --etherscan-api-key $etherscan_api_key \
            -vvv \
            $deploy_script
    else
        forge script \
            --rpc-url $eth_endpoint_http \
            --broadcast \
            -vvv \
            $deploy_script
    fi
    popd
}

deploy_gateway_logic()
{
    deploy_command scripts/DeployLocalGatewayLogic.sol:DeployLocalGatewayLogic

    pushd "$test_helpers_dir"
    pnpm generateContracts "$output_dir/contracts.json"
    popd

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

deploy_contracts()
{
    deploy_command scripts/DeployLocal.sol:DeployLocal

    pushd "$test_helpers_dir"
    pnpm generateContracts "$output_dir/contracts.json"
    popd

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

if [ -z "${from_start_services:-}" ]; then
    echo "Deploying contracts"
    deploy_contracts
fi
