#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

configure_contracts()
{
    echo "Configuring contracts"
    pushd "$ethereum_dir"

    RELAYCHAIN_ENDPOINT="ws://localhost:9944" npx hardhat run ./scripts/configure-beefy.ts --network $eth_network

    popd
}

# configure_contracts
