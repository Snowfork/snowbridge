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

build_relaychain() {
    if [ ! -f "$relaychain_bin" ]; then
        echo "Building polkadot binary as $relaychain_bin"
        rebuild_relaychain
    fi
    mkdir -p $output_bin_dir && cp "$relaychain_bin" "$output_bin_dir"/polkadot
}

rebuild_cumulus(){
    pushd $root_dir/parachain
    mkdir -p $cumulus_dir
    cargo install \
        --git https://github.com/Snowfork/cumulus \
        --branch "$cumulus_version" polkadot-parachain-bin \
        --locked \
        --root $cumulus_dir #add version path to root to avoid recompiling when switch between versions
    popd
}

build_cumulus_from_source(){
    pushd $root_dir/cumulus
    if [[ "$active_spec" == "minimal" ]]; then
      cargo build --features "minimal" --release --bin polkadot-parachain
    else
      cargo build --release --bin polkadot-parachain
    fi
    cp target/release/polkadot-parachain $output_bin_dir/polkadot-parachain
    popd
}

rebuild_relaychain(){
    pushd $root_dir/parachain
    mkdir -p $relaychain_dir
    cargo install \
        --git https://github.com/paritytech/polkadot \
        --tag "$relaychain_version" polkadot \
        --locked \
        --root $relaychain_dir #add version path to root to avoid recompiling when switch between versions
    popd
}

build_relayer()
{
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

build_query_tool() {
    echo "Building query tool"
    cargo build \
        --manifest-path tools/query-events/Cargo.toml \
        --release --features bridgehub-rococo-local \
        --bin snowbridge-query-events
    cp "$parachain_dir/target/release/snowbridge-query-events" "$output_bin_dir"
}

install_binary() {
    echo "Building and installing binaries."
    mkdir -p $output_bin_dir
    build_cumulus_from_source
    build_relaychain
    build_relayer
    build_query_tool
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
