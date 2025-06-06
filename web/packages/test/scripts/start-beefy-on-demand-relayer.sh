#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer() {
    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyClient)" \
        --arg k2 "$(address_for GatewayProxy)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .sink.contracts.BeefyClient = $k1
    | .sink.contracts.Gateway = $k2
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    ' \
        config/beefy-relay.json >$output_dir/beefy-relay.json
}

start_relayer() {
    echo "Starting relay services"
    # Launch beefy on demand relay
    (
        : >"$output_dir"/on-demand-beefy-relay.log
        while :; do
            echo "Starting on demand beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                --on-demand \
                >>"$output_dir"/on-demand-beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &
}

build_relayer() {
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

deploy_relayer() {
    check_tool && build_relayer && config_relayer && start_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "start relayers only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_relayer
    wait
fi
