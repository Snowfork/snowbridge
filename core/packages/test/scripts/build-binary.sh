#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_relaychain() {
    if [ ! -f "$relaychain_bin" ]; then
        echo "Building polkadot binary as $relaychain_bin"
        rebuild_relaychain
    fi
    cp "$relaychain_bin" "$output_bin_dir"/polkadot
}

rebuild_relaychain(){
    pushd $parachain_dir
    cargo install \
        --git https://github.com/paritytech/polkadot \
        --tag "$relaychain_version" polkadot \
        --locked \
        --root .cargo
    mkdir -p "$(dirname "$relaychain_bin")"
    cp "$parachain_dir"/.cargo/bin/polkadot "$relaychain_bin" || true
    popd
}

build_parachain()
{
    if [ "$eth_network" != "localhost" ]; then
        parachain_runtime="snowblink"
    fi

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

    echo "Building test parachain"
    cargo build \
        --manifest-path utils/test-parachain/Cargo.toml \
        --release \
        --bin snowbridge-test-node
    cp "$test_collator_bin" "$output_bin_dir"

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
    build_relaychain
    build_parachain
    build_relayer
}
