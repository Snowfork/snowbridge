#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_relaychain() {
    # if [ ! -d "$relaychain_dir" ] ; then
    #     echo "clone polkadot project to $relaychain_dir"
    #     git clone https://github.com/paritytech/polkadot.git $relaychain_dir
    # fi
    # if [ ! -f "$relaychain_bin" ]; then
    #     echo "Building polkadot binary as $relaychain_bin"
    #     rebuild_relaychain
    # fi
    # cp "$relaychain_bin" "$output_bin_dir"
    true;
}

rebuild_relaychain(){
    # pushd $relaychain_dir
    # git fetch --tags
    # git checkout $relaychain_version
    # cargo build --release
    # popd
    true;
}

build_parachain()
{
    if [ "$eth_network" != "localhost" ]; then
        parachain_runtime="snowblink"
    fi

    echo "Runtime is $parachain_runtime"

    echo "Building snowbridge parachain"
    cargo build \
        --manifest-path "$parachain_dir/Cargo.toml" \
        --release \
        --no-default-features \
        --features "${parachain_runtime}-native,rococo-native" \
        --bin snowbridge
    cp "$parachain_dir/target/release/snowbridge" "$output_bin_dir"

    echo "Building query tool"
    cargo build --manifest-path "$parachain_dir/tools/query-events/Cargo.toml" --release --features parachain-snowbase --bin snowbridge-query-events
    cp "$parachain_dir/target/release/snowbridge-query-events" "$output_bin_dir"

    echo "Building test parachain"
    cargo build --manifest-path "$parachain_dir/utils/test-parachain/Cargo.toml" --release --bin snowbridge-test-node
    cp "$test_collator_bin" "$output_bin_dir"
}

build_relayer()
{
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

build_e2e_test() {
    echo "Building tests"
    pushd "$core_dir"
    pnpm install
    popd
}

install_binary() {
    echo "Building and installing binaries."
    build_relaychain
    build_parachain
    build_relayer
    build_e2e_test
}
