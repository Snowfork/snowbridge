#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

fund_gateway() {
    pushd "$contract_dir"
    forge script \
        --rpc-url $eth_endpoint_http \
        --broadcast \
        -vvv \
        scripts/FundGateway.sol:FundGateway
    popd

    echo "Fund gateway success!"
}

if [ -z "${from_start_services:-}" ]; then
    echo "Funding gateway"
    fund_gateway
fi
