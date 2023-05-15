#!/usr/bin/env bash
set -eu

source scripts/set-env.sh


wait_start() {
    scripts/wait-for-it.sh -t 120 127.0.0.1:12144
}

zombienet_launch() {
    npx zombienet spawn config/launch-config.toml --provider=native 2>&1 &
    wait_start
}

deploy_polkadot() {
    zombienet_launch

    # Configure statemine
    scripts/configure-statemine.sh add-exporter-config \
        $relaychain_ws_url \
        $relaychain_sudo_seed \
        $statemine_para_id \
        $statemine_ws_url \
        $bridgehub_para_id \
        $eth_chain_id
}
