#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_polkadot_binaries() {
    pushd $root_dir
    pushd $polkadot_sdk_dir

    local features=''
    if [ "$eth_network" == "localhost" ]; then
        features="--features fast-runtime"
    fi

    check_local_changes "polkadot"
    check_local_changes "substrate"

    # Check that all 3 binaries are available and no changes made in the polkadot and substrate dirs
    if [[ ! -e "target/release/polkadot" || ! -e "target/release/polkadot-execute-worker" || ! -e "target/release/polkadot-prepare-worker" || "$changes_detected" -eq 1 ]]; then
        echo "Building polkadot binary, due to changes detected in polkadot or substrate, or binaries not found"
        cargo build --release --locked --bin polkadot --bin polkadot-execute-worker --bin polkadot-prepare-worker $features
    else
        echo "No changes detected in polkadot or substrate and binaries are available, not rebuilding relaychain binaries."
    fi

    mkdir -p "$output_bin_dir"

    cp target/release/polkadot $output_bin_dir/polkadot
    cp target/release/polkadot-execute-worker $output_bin_dir/polkadot-execute-worker
    cp target/release/polkadot-prepare-worker $output_bin_dir/polkadot-prepare-worker

    echo "Building polkadot-parachain binary"
    cargo build --release --locked -p polkadot-parachain-bin --bin polkadot-parachain $features
    cp target/release/polkadot-parachain $output_bin_dir/polkadot-parachain

    popd
    popd
}

changes_detected=0

check_local_changes() {
    local dir=$1
    cd "$dir"
    if git status --untracked-files=no --porcelain . | grep .; then
        changes_detected=1
    fi
    cd -
}

build_contracts() {
    echo "Building contracts"
    pushd $root_dir/contracts
    forge build
    popd
}

build_relayer() {
    echo "Building relayer v2"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir/snowbridge-relay-v2"
}

set_slot_time() {
    local new_value=$1
    echo "Hack lodestar for faster slot time"
    local preset_mainnet_config_file="$root_dir/../lodestar/packages/config/src/chainConfig/configs/mainnet.ts"
    if [[ "$(uname)" == "Darwin" && -z "${IN_NIX_SHELL:-}" ]]; then
        gsed -i "s/SECONDS_PER_SLOT: .*/SECONDS_PER_SLOT: $new_value,/g" $preset_mainnet_config_file
    else
        sed -i "s/SECONDS_PER_SLOT: .*/SECONDS_PER_SLOT: $new_value,/g" $preset_mainnet_config_file
    fi
}

build_lodestar() {
    if [ "$rebuild_lodestar" == "true" ]; then
        pushd $root_dir/../lodestar
        if [ "$eth_fast_mode" == "true" ]; then
            set_slot_time 1
        else
            set_slot_time 12
        fi
        yarn install && yarn run build
        popd
    fi
}

build_web_packages() {
    if [ "$rebuild_web_packages" == "true" ]; then
        pushd $root_dir/web
        pnpm install
        pnpm build
        popd
    fi
}

build_v1() {
    if [ ! -d $v1_root_dir ]; then
        git clone -b snowbridge-v1 https://github.com/snowfork/snowbridge/ $v1_root_dir
    fi
    pushd $v1_contract_dir
    forge build
    popd
    pushd $v1_relay_dir
    mage build
    cp $v1_relay_dir/build/snowbridge-relay "$output_bin_dir/snowbridge-relay-v1"
    popd
}

install_binary() {
    echo "Building and installing binaries."
    mkdir -p $output_bin_dir
    build_lodestar
    build_polkadot_binaries
    build_contracts
    build_relayer
    build_web_packages
    build_v1
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
