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
    pushd $parachain_dir
    mkdir -p $cumulus_dir
    cargo install \
        --git https://github.com/paritytech/cumulus \
        --tag "$cumulus_version" polkadot-parachain \
        --locked \
        --root $cumulus_dir #add version path to root to avoid recompiling when switch between versions 
    popd
}

rebuild_relaychain(){
    pushd $parachain_dir
    mkdir -p $relaychain_dir
    cargo install \
        --git https://github.com/paritytech/polkadot \
        --tag "$relaychain_version" polkadot \
        --locked \
        --root $relaychain_dir #add version path to root to avoid recompiling when switch between versions 
    popd
}

# Only for debug purpose when we need to do some customization in relaychain
build_relaychain_from_source(){
    relaychain_src_dir="$relaychain_dir/src"
    if [ ! -d "$relaychain_src_dir" ] ; then
        echo "clone polkadot project to $relaychain_src_dir"
        git clone https://github.com/paritytech/polkadot.git $relaychain_src_dir
    fi
    pushd $relaychain_src_dir
    git switch release-$relaychain_version
    cargo build --release
    cp "$relaychain_src_dir/target/release/polkadot" "$output_bin_dir"
    popd
}

build_parachain()
{
    echo "Runtime is $parachain_runtime"

    echo "Building snowbridge parachain"
    cd $parachain_dir

    cargo build \
        --manifest-path Cargo.toml \
        --release \
        --no-default-features \
        --features "${parachain_runtime}-native,rococo-native" \
        --bin snowbridge
    cp "$parachain_dir/target/release/snowbridge" "$output_bin_dir"

    echo "Building query tool"
    cargo build \
        --manifest-path tools/query-events/Cargo.toml \
        --release --features parachain-snowbase \
        --bin snowbridge-query-events
    cp "$parachain_dir/target/release/snowbridge-query-events" "$output_bin_dir"

    cd -
}

build_relayer()
{
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

install_binary() {
    echo "Building and installing binaries."
    build_cumulus
    build_relaychain
    build_parachain
    build_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
