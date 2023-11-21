#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_binaries() {
    pushd $root_dir/polkadot-sdk

    local features=''
    if [[ "$active_spec" != "minimal" ]]; then
        features=--features beacon-spec-mainnet
    fi

    echo "Building polkadot binary and parachain template node"
    cargo build --release --workspace --locked --bin polkadot --bin polkadot-execute-worker --bin polkadot-prepare-worker --bin parachain-template-node
    cp target/release/polkadot $output_bin_dir/polkadot
    cp target/release/polkadot-execute-worker $output_bin_dir/polkadot-execute-worker
    cp target/release/polkadot-prepare-worker $output_bin_dir/polkadot-prepare-worker
    cp target/release/parachain-template-node $output_bin_dir/parachain-template-node

    echo "Building polkadot-parachain binary"
    cargo build --release --workspace --locked --bin polkadot-parachain $features
    cp target/release/polkadot-parachain $output_bin_dir/polkadot-parachain

    popd
}

build_contracts() {
    echo "Building contracts"
    pushd $root_dir/contracts
    forge build
    popd
}

build_relayer() {
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

hack_beacon_client() {
    echo "Hack lodestar for faster slot time"
    local preset_minimal_config_file="$root_dir/lodestar/packages/config/src/chainConfig/presets/minimal.ts"
    if [[ "$(uname)" == "Darwin" && -z "${IN_NIX_SHELL:-}" ]]; then
        gsed -i "s/SECONDS_PER_SLOT: 6/SECONDS_PER_SLOT: 2/g" $preset_minimal_config_file
    else
        sed -i "s/SECONDS_PER_SLOT: 6/SECONDS_PER_SLOT: 2/g" $preset_minimal_config_file
    fi
}

build_lodestar() {
    if [ ! -d "$root_dir/lodestar/packages/cli/lib" ]; then
        pushd $root_dir/lodestar
        if [ "$eth_fast_mode" == "true" ]; then
            hack_beacon_client
        fi
        yarn install && yarn run build
        popd
    else
        echo "lodestar has already been built."
    fi
}

build_geth() {
    pushd $root_dir/go-ethereum
    make geth
    cp build/bin/geth "$output_bin_dir"
    popd
}

install_binary() {
    echo "Building and installing binaries."
    mkdir -p $output_bin_dir
    build_lodestar
    build_geth
    build_binaries
    build_contracts
    build_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
