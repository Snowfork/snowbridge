#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

configure_base_fee() {
    pushd $root_dir/smoketest
    ./make-bindings.sh
    cargo test --test configure_base_fee
    popd
}

if [ -z "${from_start_services:-}" ]; then
    echo "config base fee!"
    configure_base_fee
    wait
fi
