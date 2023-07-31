#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh

zombienet_launch() {
    npx zombienet spawn config/launch-config.toml --provider=native --dir="$zombienet_data_dir" 2>&1 &
    scripts/wait-for-it.sh -t 120 127.0.0.1:12144
}

deploy_polkadot() {
    check_tool && build_relaychain && build_cumulus_from_source && rm -rf $zombienet_data_dir && zombienet_launch
}

if [ -z "${from_start_services:-}" ]; then
    echo "start polkadot only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_polkadot
    echo "polkadot nodes started"
    wait
fi
