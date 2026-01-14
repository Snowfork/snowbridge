#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer() {
    local electra_forked_epoch=0
    local fulu_forked_epoch=50000000
    # Configure beefy relay 1 (uses wrapper address)
    jq \
        --arg k1 "$(address_for BeefyClientWrapperProxy)" \
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

    # Configure beefy relay 2 (uses wrapper address)
    jq \
        --arg k1 "$(address_for BeefyClientWrapperProxy)" \
        --arg k2 "$(address_for GatewayProxy)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .sink.contracts.BeefyClient = $k1
    | .sink.contracts.Gateway = $k2
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    ' \
        config/beefy-relay.json >$output_dir/beefy-relay-2.json

    # Configure parachain relay v1
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg eth_gas_limit $eth_gas_limit \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_writer_endpoint
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
        config/parachain-relay-v1.json >$output_dir/parachain-relay-v1.json

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
        config/parachain-relay-v1.json >$output_dir/parachain-relay-bridge-hub-01.json

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
        config/parachain-relay-v1.json >$output_dir/parachain-relay-bridge-hub-02.json

    # Configure parachain relay v2
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
        config/parachain-relay.json >$output_dir/parachain-relay.json

    # Configure fisherman relay
    jq \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_writer_endpoint $eth_writer_endpoint \
        --arg eth_gas_limit $eth_gas_limit \
        '
      .source.contracts.BeefyClient = $k2
    | .source.ethereum.endpoint = $eth_endpoint_ws
    ' \
        config/fisherman-relay.json >$output_dir/fisherman-relay.json

    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --argjson electra_forked_epoch $electra_forked_epoch \
        --argjson fulu_forked_epoch $fulu_forked_epoch \
        '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    | .source.beacon.spec.forkVersions.fulu = $fulu_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json

    # Configure execution relay v1
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --argjson electra_forked_epoch $electra_forked_epoch \
        --argjson fulu_forked_epoch $fulu_forked_epoch \
        --arg channelID $ASSET_HUB_CHANNEL_ID \
        '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .schedule.id = 0
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    | .source.beacon.spec.forkVersions.fulu = $fulu_forked_epoch
    | .source."channel-id" = $channelID

    ' \
        config/execution-relay-v1.json >$output_dir/execution-relay-v1.json

    # Configure execution relay v2
      jq \
          --arg eth_endpoint_ws $eth_endpoint_ws \
          --arg k1 "$(address_for GatewayProxy)" \
          --argjson electra_forked_epoch $electra_forked_epoch \
          --argjson fulu_forked_epoch $fulu_forked_epoch \
          '
        .source.ethereum.endpoint = $eth_endpoint_ws
      | .source.contracts.Gateway = $k1
      | .schedule.id = 0
      | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
      | .source.beacon.spec.forkVersions.fulu = $fulu_forked_epoch

      ' \
          config/execution-relay.json >$output_dir/execution-relay.json

    # Configure reward relay
      jq \
          --arg eth_endpoint_ws $eth_endpoint_ws \
          --arg k1 "$(address_for GatewayProxy)" \
          --argjson electra_forked_epoch $electra_forked_epoch \
          --argjson fulu_forked_epoch $fulu_forked_epoch \
          '
        .source.ethereum.endpoint = $eth_endpoint_ws
      | .source.contracts.Gateway = $k1
      | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
      | .source.beacon.spec.forkVersions.fulu = $fulu_forked_epoch

      ' \
          config/reward-relay.json >$output_dir/reward-relay.json
}

start_relayer() {
    echo "Starting relay services"
    # Launch beefy relay 1
    (
        : >"$output_dir"/beefy-relay.log
        while :; do
            echo "Starting beefy relay 1 at $(date)"
            "${relayer_v2}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                >>"$output_dir"/beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beefy relay 2
    (
        : >"$output_dir"/beefy-relay-2.log
        while :; do
            echo "Starting beefy relay 2 at $(date)"
            "${relayer_v2}" run beefy \
                --config "$output_dir/beefy-relay-2.json" \
                --ethereum.private-key $beefy_relay_eth_key_2 \
                >>"$output_dir"/beefy-relay-2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay v1
    (
        : >"$output_dir"/parachain-relay-v1.log
        while :; do
            echo "Starting parachain relay v1 at $(date)"
            "${relayer_v1}" run parachain \
                --config "$output_dir/parachain-relay-v1.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                >>"$output_dir"/parachain-relay-v1.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay v2
    (
        : >"$output_dir"/parachain-relay-v2.log
        while :; do
            echo "Starting parachain relay v2 at $(date)"
            "${relayer_v2}" run parachain \
                --config "$output_dir/parachain-relay.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/parachain-relay-v2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch equivocation fisherman
    (
        : >"$output_dir"/equivocation-fisherman.log
        while :; do
            echo "Starting equivocation fisherman at $(date)"
            "${relayer_v2}" run fisherman \
                --config "$output_dir/fisherman-relay.json" \
                --ethereum.private-key $parachain_relay_primary_gov_eth_key \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/equivocation-fisherman.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beacon relay
    (
        : >"$output_dir"/beacon-relay.log
        while :; do
            echo "Starting beacon relay at $(date)"
            "${relayer_v2}" run beacon \
                --config $output_dir/beacon-relay.json \
                --substrate.private-key "//BeaconRelay" \
                >>"$output_dir"/beacon-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay v1
    (
        : >$output_dir/execution-relay-v1.log
        while :; do
            echo "Starting execution relay v1 at $(date)"
            "${relayer_v1}" run execution \
                --config $output_dir/execution-relay-v1.json \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/execution-relay-v1.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay v2
    (
        : >$output_dir/execution-relay-v2.log
        while :; do
            echo "Starting execution relay v2 at $(date)"
            "${relayer_v2}" run execution \
                --config $output_dir/execution-relay.json \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/execution-relay-v2.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch reward relay
    (
        : >$output_dir/reward-relay.log
        while :; do
            echo "Starting reward relay at $(date)"
            "${relayer_v2}" run reward \
                --config $output_dir/reward-relay.json \
                --substrate.private-key "//ExecutionRelayAssetHub" \
                >>"$output_dir"/reward-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay for bridgehub (primary governance)
    (
        : >"$output_dir"/parachain-relay-bridge-hub-01.log
        while :; do
            echo "Starting parachain-relay (primary governance) at $(date)"
            "${relayer_v1}" run parachain \
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
            "${relayer_v1}" run parachain \
                --config "$output_dir/parachain-relay-bridge-hub-02.json" \
                --ethereum.private-key $parachain_relay_secondary_gov_eth_key \
                >>"$output_dir"/parachain-relay-bridge-hub-02.log 2>&1 || true
            sleep 20
        done
    ) &
}

build_relayer() {
    echo "Building relayer v2"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir/snowbridge-relay-v2"
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
