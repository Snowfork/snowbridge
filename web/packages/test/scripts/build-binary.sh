#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_cumulus() {
    if [ ! -f "$cumulus_bin" ]; then
        echo "Building cumulus binary as $cumulus_bin"
        rebuild_cumulus
    fi
    mkdir -p $output_bin_dir && cp "$cumulus_bin" "$output_bin_dir"/polkadot-parachain
}

rebuild_cumulus() {
    pushd $root_dir/parachain
    mkdir -p $cumulus_dir
    cargo install \
        --git https://github.com/Snowfork/cumulus \
        --branch "$cumulus_version" polkadot-parachain-bin \
        --locked \
        --root $cumulus_dir #add version path to root to avoid recompiling when switch between versions
    popd
}

build_cumulus_from_source() {
    pushd $root_dir/cumulus
    if [[ "$active_spec" == "minimal" ]]; then
        cargo build --release --bin polkadot-parachain
    else
        cargo build --features beacon-spec-mainnet --release --bin polkadot-parachain
    fi
    cp target/release/polkadot-parachain $output_bin_dir/polkadot-parachain
    cargo build --release --locked --bin parachain-template-node
    cp target/release/parachain-template-node $output_bin_dir/parachain-template-node
    popd
}

build_relaychain() {
    if [ ! -f "$relaychain_bin" ]; then
        echo "Building polkadot binary as $relaychain_bin"
        rebuild_relaychain
    fi
    mkdir -p $output_bin_dir && cp "$relaychain_bin" "$output_bin_dir"/polkadot
}

rebuild_relaychain() {
    pushd $root_dir/parachain
    mkdir -p $relaychain_dir
    cargo install \
        --git https://github.com/paritytech/polkadot \
        --tag "$relaychain_version" polkadot \
        --locked \
        --root $relaychain_dir #add version path to root to avoid recompiling when switch between versions
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
    pushd $root_dir/lodestar
    if [ "$eth_fast_mode" == "true" ]; then
        hack_beacon_client
    fi
    yarn install && yarn build
    popd
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
    build_cumulus_from_source
    build_relaychain
    build_lodestar
    build_geth
    build_contracts
    build_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
