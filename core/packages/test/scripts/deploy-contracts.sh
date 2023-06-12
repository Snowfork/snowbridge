#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

deploy_contracts()
{
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
    node scripts/generateContractInfo.js "$output_dir/contracts.json"
    popd
    echo "Exported contract artifacts: $output_dir/contracts.json"
    if [ "$eth_network" != "localhost" ]; then
      timestamp=$(date +%s)
      commit_id=$(git rev-parse --verify HEAD)
      cp "$output_dir"/contracts.json deployment/"$eth_network"-"$commit_id"-"$timestamp".json
    fi
}

if [ -z "${from_start_services:-}" ]; then
    echo "Deploying contracts"
    deploy_contracts
fi
