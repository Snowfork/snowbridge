#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer() {
    # Configure parachain relay
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-bridge-hub-01.json

}

start_relayer() {
    echo "Starting relay services"

    # Launch parachain relay
    (
        : >"$output_dir"/parachain-relay-bridge-hub-01.log
        while :; do
            echo "Starting parachain-relay at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-bridge-hub-01.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/parachain-relay-bridge-hub-01.log 2>&1 || true
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
