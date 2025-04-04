#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

build_binaries() {
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
        EPOCH_DURATION=10 cargo build --release --locked --bin polkadot --bin polkadot-execute-worker --bin polkadot-prepare-worker $features
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

build_latest_relayer() {
    echo "Building latest relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

build_relayers_v1_v2() {
    pushd "$root_dir"

    # Backup relayer directory
    BACKUP_DIR=$(mktemp -d -t relayer-backup-XXXXXX)
    echo "Backing up relayer directory to $BACKUP_DIR"
    cp -r "$relay_dir" "$BACKUP_DIR/"

    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

    # Build current version
    echo "Building relayer v2"
    checkout_build_and_copy "$CURRENT_BRANCH" "snowbridge-relay-v2"

    # Build snowbridge-v1 branch version
    echo "Building relayer v1"
    checkout_build_and_copy "snowbridge-v1" "snowbridge-relay-v1"

    # Restore original relayer directory
    echo "Restoring original relayer directory from backup"
    rm -rf "$relay_dir"
    mv "$BACKUP_DIR/$(basename "$relay_dir")" "$relay_dir"
    rm -rf "$BACKUP_DIR"

    popd
}

# Function to checkout, build relayer, and copy binary
checkout_build_and_copy() {
    BRANCH=$1
    BINARY_NAME=$2

    pushd $root_dir

    rm -rf "$relay_dir"

    echo "Checking out relayer directory from branch: $BRANCH"
    git checkout "$BRANCH" -- "relayer"

    echo "Building relayer from branch: $BRANCH"
    mage -d "$relay_dir" build

    echo "Copying binary to output directory"
    cp $relay_bin "$output_bin_dir/$BINARY_NAME"
    popd
}

set_slot_time() {
    local new_value=$1
    echo "Hack lodestar for faster slot time"
    local preset_mainnet_config_file="$root_dir/lodestar/packages/config/src/chainConfig/configs/mainnet.ts"
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
    pushd $root_dir/web
    pnpm --version
    pnpm install
    pnpm build
    popd
}

install_binary() {
    echo "Building and installing binaries."
    mkdir -p $output_bin_dir
    build_lodestar
    build_binaries
    build_contracts
    build_web_packages
    if [ "$snowbridge_v1_v2" = true ]; then
        build_relayers_v1_v2
    else
        build_latest_relayer
    fi
}

if [ -z "${from_start_services:-}" ]; then
    echo "build binaries only!"
    install_binary
fi
