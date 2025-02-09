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
        --arg assetHubChannelID $ASSET_HUB_CHANNEL_ID \
        '
      .sink.contracts.BeefyClient = $k1
    | .sink.contracts.Gateway = $k2
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | ."on-demand-sync"."asset-hub-channel-id" = $assetHubChannelID
    ' \
        config/beefy-relay.json >$output_dir/beefy-relay.json


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
        config/parachain-relay-v2.json >$output_dir/parachain-relay-v2.json

    # Configure beacon relay
    local deneb_forked_epoch=132608
    if [ "$eth_fast_mode" == "true" ]; then
        deneb_forked_epoch=0
    fi
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --argjson deneb_forked_epoch $deneb_forked_epoch \
        '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.spec.denebForkedEpoch = $deneb_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json

    # Configure execution relay
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .source."channel-id" = $channelID
    | .schedule.id = 0
    ' \
        config/execution-relay.json >$output_dir/execution-relay-asset-hub-0.json
}

start_relayer() {
    echo "Starting relay services"
    # Launch beefy relay
    (
        : >"$output_dir"/beefy-relay.log
        while :; do
            echo "Starting beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                >>"$output_dir"/beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay
    (
        : >"$output_dir"/parachain-relay-v2.log
        while :; do
            echo "Starting parachain-relay-v2 at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-v2.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/parachain-relay-v2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beacon relay
    (
        : >"$output_dir"/beacon-relay.log
        while :; do
            echo "Starting beacon relay at $(date)"
            "${relay_bin}" run beacon \
                --config $output_dir/beacon-relay.json \
                --substrate.private-key "//BeaconRelay" \
                >>"$output_dir"/beacon-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay
    (
        : >$output_dir/execution-relay-v2.log
        while :; do
            echo "Starting execution relay at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay-asset-hub-0.json \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/execution-relay-v2.log 2>&1 || true
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
