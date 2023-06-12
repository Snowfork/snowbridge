#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer(){
    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_gas_limit $eth_gas_limit \
    '
      .sink.contracts.BeefyClient = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    ' \
    config/beefy-relay.json > $output_dir/beefy-relay.json

    # Configure parachain relay
    jq \
        --arg k1 "$(address_for InboundQueue)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg laneID $ASSET_HUB_PARAID \
        --arg eth_gas_limit $eth_gas_limit \
    '
      .source.contracts.InboundQueue = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.InboundQueue = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."lane-id" = $laneID
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay.json

    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --arg active_spec $active_spec \
    '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.activeSpec = $active_spec
    ' \
    config/beacon-relay.json > $output_dir/beacon-relay.json

    # Configure execution relay
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for OutboundQueue)" \
        --arg laneID $ASSET_HUB_PARAID \
    '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.OutboundQueue = $k1
    | .source."lane-id" = $laneID
    ' \
    config/execution-relay.json > $output_dir/execution-relay.json
}

start_relayer()
{
    echo "Starting relay services"
    # Launch beefy relay
    (
        : > "$output_dir"/beefy-relay.log
        while :
        do
            echo "Starting beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                >> "$output_dir"/beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay
    (
        : > "$output_dir"/parachain-relay.log
        while :
        do
          echo "Starting parachain relay at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay.json" \
                --ethereum.private-key $parachain_relay_eth_key \
                >> "$output_dir"/parachain-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beacon relay
    (
        : > "$output_dir"/beacon-relay.log
        while :
        do
        echo "Starting beacon relay at $(date)"
            "${relay_bin}" run beacon \
                --config $output_dir/beacon-relay.json \
                --substrate.private-key "//BeaconRelay" \
                >> "$output_dir"/beacon-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay
    (
        : > $output_dir/execution-relay.log
        while :
        do
        echo "Starting execution relay at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay.json \
                --substrate.private-key "//ExecutionRelay" \
                >> "$output_dir"/execution-relay.log 2>&1 || true
            sleep 20
        done
    ) &
}

build_relayer()
{
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

deploy_relayer()
{
    check_tool && build_relayer && config_relayer && start_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "start relayers only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_relayer
    wait
fi
