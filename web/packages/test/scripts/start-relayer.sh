#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer(){
    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg eth_gas_limit $eth_gas_limit \
        --arg relaychain_ws_url $relaychain_ws_url \
    '
      .sink.contracts.BeefyClient = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.polkadot.endpoint = $relaychain_ws_url
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    ' \
    config/beefy-relay.json > $output_dir/beefy-relay.json

    # Configure parachain relay (bridge hub)
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg channelID $BRIDGE_HUB_PARAID \
        --arg eth_gas_limit $eth_gas_limit \
        --arg relaychain_ws_url $relaychain_ws_url \
        --arg bridgehub_ws_url $bridgehub_ws_url \
    '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.polkadot.endpoint = $relaychain_ws_url
    | .source.parachain.endpoint = $bridgehub_ws_url
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay-bridge-hub.json

    # Configure parachain relay (asset hub)
    jq \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg k2 "$(address_for BeefyClient)" \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg channelID $ASSET_HUB_PARAID \
        --arg eth_gas_limit $eth_gas_limit \
        --arg relaychain_ws_url $relaychain_ws_url \
        --arg bridgehub_ws_url $bridgehub_ws_url \
    '
      .source.contracts.Gateway = $k1
    | .source.contracts.BeefyClient = $k2
    | .sink.contracts.Gateway = $k1
    | .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.polkadot.endpoint = $relaychain_ws_url
    | .source.parachain.endpoint = $bridgehub_ws_url
    | .sink.ethereum.endpoint = $eth_endpoint_ws
    | .sink.ethereum."gas-limit" = $eth_gas_limit
    | .source."channel-id" = $channelID
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay-asset-hub.json

    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --arg active_spec $active_spec \
        --arg bridgehub_ws_url $bridgehub_ws_url \
    '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.activeSpec = $active_spec
    | .sink.parachain.endpoint = $bridgehub_ws_url
    ' \
    config/beacon-relay.json > $output_dir/beacon-relay.json

    # Configure execution relay
    jq \
        --arg eth_endpoint_ws $eth_endpoint_ws \
        --arg k1 "$(address_for GatewayProxy)" \
        --arg channelID $ASSET_HUB_PARAID \
        --arg bridgehub_ws_url $bridgehub_ws_url \
    '
      .source.ethereum.endpoint = $eth_endpoint_ws
    | .source.contracts.Gateway = $k1
    | .source."channel-id" = $channelID
    | .sink.parachain.endpoint = $bridgehub_ws_url
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

    # Launch parachain relay for bridgehub
    (
        : > "$output_dir"/parachain-relay-bridge-hub.log
        while :
        do
          echo "Starting parachain-relay (bridgehub) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-bridge-hub.json" \
                --ethereum.private-key $parachain_relay_eth_key \
                >> "$output_dir"/parachain-relay-bridge-hub.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay for statemint
    (
        : > "$output_dir"/parachain-relay-asset-hub.log
        while :
        do
          echo "Starting parachain relay (asset-hub) at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay-asset-hub.json" \
                --ethereum.private-key $parachain_relay_eth_key \
                >> "$output_dir"/parachain-relay-asset-hub.log 2>&1 || true
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

docker_start_relayer()
{
    docker-compose up -d
}

docker_stop_relayer()
{
    docker-compose down
}

build_relayer()
{
    echo "Building relayer"
    mage -d "$relay_dir" build
    cp $relay_bin "$output_bin_dir"
}

deploy_relayer()
{
    check_tool && build_relayer && config_relayer && start_relayer # docker_start_relayer
}

if [ -z "${from_start_services:-}" ]; then
    echo "start relayers only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_relayer
    wait
fi
