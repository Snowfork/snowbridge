#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

configure_contracts()
{
    pushd "$ethereum_dir"

    npx hardhat run ./scripts/configure-beefy.ts --network $eth_network

    popd
}
