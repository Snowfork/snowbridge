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

install_binary() {
    echo "Building and installing binaries."
    mkdir -p $output_bin_dir
    build_binaries
    build_contracts
    build_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
