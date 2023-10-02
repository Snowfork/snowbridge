#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_cumulus_from_source() {
    pushd $root_dir/polkadot-sdk/cumulus

    local features=''
    if [[ "$active_spec" != "minimal" ]]; then
        features=--features beacon-spec-mainnet
    fi

    echo "Building polkadot-parachain binary"
    cargo build --release --workspace --locked --bin polkadot-parachain $features
    cp ../target/release/polkadot-parachain $output_bin_dir/polkadot-parachain

    echo "Building parachain template node"
    cargo build --release --workspace --locked --bin parachain-template-node
    cp ../target/release/parachain-template-node $output_bin_dir/parachain-template-node
    popd
}

build_relaychain_from_source() {
    echo "Building polkadot binary"
    pushd $root_dir/polkadot-sdk

    cargo build --release --locked --bin polkadot --bin polkadot-execute-worker --bin polkadot-prepare-worker
    mkdir -p $output_bin_dir

    cp target/release/polkadot $output_bin_dir/polkadot
    cp target/release/polkadot-execute-worker $output_bin_dir/polkadot-execute-worker
    cp target/release/polkadot-prepare-worker $output_bin_dir/polkadot-prepare-worker

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
    build_cumulus_from_source
    build_relaychain_from_source
    build_contracts
    build_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
