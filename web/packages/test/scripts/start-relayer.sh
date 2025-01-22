#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer() {
    local electra_forked_epoch=2000000
    if [ "$is_electra" == "true" ]; then
        electra_forked_epoch=0
    fi
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

    # Configure parachain relay (primary governance)
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $PRIMARY_GOVERNANCE_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-bridge-hub-01.json

    # Configure parachain relay (secondary governance)
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $SECONDARY_GOVERNANCE_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-bridge-hub-02.json

    # Configure parachain relay (asset hub)-0
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    | .schedule.id = 0
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-asset-hub-0.json

    # Configure parachain relay (asset hub)-1
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    | .schedule.id = 1
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-asset-hub-1.json

    # Configure parachain relay (asset hub)-2
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    | .schedule.id = 2
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-asset-hub-2.json

    # Configure parachain relay (penpal)
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg channelID $PENPAL_CHANNEL_ID \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
        config/parachain-relay.json >$output_dir/parachain-relay-penpal.json

    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json

    # Configure execution relay for assethub-0
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .source."channel-id" = $channelID
    | .schedule.id = 0
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/execution-relay.json >$output_dir/execution-relay-asset-hub-0.json

    # Configure execution relay for assethub-1
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .source."channel-id" = $channelID
    | .schedule.id = 1
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/execution-relay.json >$output_dir/execution-relay-asset-hub-1.json

    # Configure execution relay for assethub-2
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .source."channel-id" = $channelID
    | .schedule.id = 2
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/execution-relay.json >$output_dir/execution-relay-asset-hub-2.json

    # Configure execution relay for penpal
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $PENPAL_CHANNEL_ID \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
              .source.ethereum.endpoint = $eth_endpoint_ws
            | .source.contracts.Gateway = $k1
            | .source."channel-id" = $channelID
            | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
            ' \
        config/execution-relay.json >$output_dir/execution-relay-penpal.json
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

    # Launch parachain relay for bridgehub (primary governance)
    (
        : >"$output_dir"/parachain-relay-bridge-hub-01.log
        while :; do
            echo "Starting parachain-relay (primary governance) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-bridge-hub-01.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                >>"$output_dir"/parachain-relay-bridge-hub-01.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay for bridgehub (secondary governance)
    (
        : >"$output_dir"/parachain-relay-bridge-hub-02.log
        while :; do
            echo "Starting parachain-relay (secondary governance) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-bridge-hub-02.json" \
                --ethereum.private-key $parachain_relay_secondary_gov_eth_key \
                >>"$output_dir"/parachain-relay-bridge-hub-02.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay 0 for assethub
    (
        : >"$output_dir"/parachain-relay-asset-hub-0.log
        while :; do
            echo "Starting parachain relay (asset-hub) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-asset-hub-0.json" \
                --ethereum.private-key $parachain_relay_assethub_eth_key \
                >>"$output_dir"/parachain-relay-asset-hub-0.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay 1 for assethub
    (
        : >"$output_dir"/parachain-relay-asset-hub-1.log
        while :; do
            echo "Starting parachain relay (asset-hub) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-asset-hub-1.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                >>"$output_dir"/parachain-relay-asset-hub-1.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay 2 for assethub
    (
        : >"$output_dir"/parachain-relay-asset-hub-2.log
        while :; do
            echo "Starting parachain relay (asset-hub) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-asset-hub-2.json" \
                --ethereum.private-key $parachain_relay_secondary_gov_eth_key \
                >>"$output_dir"/parachain-relay-asset-hub-2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay for parachain penpal
    (
        : >"$output_dir"/parachain-relay-penpal.log
        while :; do
            echo "Starting parachain-relay (penpal) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-penpal.json" \
                --ethereum.private-key $parachain_relay_penpal_eth_key \
                >>"$output_dir"/parachain-relay-penpal.log 2>&1 || true
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

    # Launch execution relay for assethub-0
    (
        : >$output_dir/execution-relay-asset-hub-0.log
        while :; do
            echo "Starting execution relay (asset-hub-0) at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay-asset-hub-0.json \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/execution-relay-asset-hub-0.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay for assethub-1
    (
        : >$output_dir/execution-relay-asset-hub-1.log
        while :; do
            echo "Starting execution relay (asset-hub-1) at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay-asset-hub-1.json \
                --substrate.private-key "//Alice" \
                >>"$output_dir"/execution-relay-asset-hub-1.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay for assethub-2
    (
        : >$output_dir/execution-relay-asset-hub-2.log
        while :; do
            echo "Starting execution relay (asset-hub-2) at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay-asset-hub-2.json \
                --substrate.private-key "//Bob" \
                >>"$output_dir"/execution-relay-asset-hub-2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay for penpal
    (
        : >$output_dir/execution-relay-penpal.log
        while :; do
            echo "Starting execution relay (penpal) at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay-penpal.json \
                --substrate.private-key "//ExecutionRelayPenpal" \
                >>"$output_dir"/execution-relay-penpal.log 2>&1 || true
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
