#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

start_relayer()
{
    echo "Starting relay services"

    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyClient)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
    '
      .sink.contracts.BeefyClient = $k1
    | .source.ethereum.endpoint = $infura_endpoint_ws
    | .sink.ethereum.endpoint = $infura_endpoint_ws
    ' \
    config/beefy-relay.json > $output_dir/beefy-relay.json

    # Configure parachain relay
    jq \
        --arg k1 "$(address_for BasicInboundChannel)" \
        --arg k2 "$(address_for IncentivizedInboundChannel)" \
        --arg k3 "$(address_for BeefyClient)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
        --arg basic_parachain_account_ids $basic_parachain_account_ids \
    '
      .source.contracts.BasicInboundChannel = $k1
    | .source.contracts.IncentivizedInboundChannel = $k2
    | .source.contracts.BeefyClient = $k3
    | .sink.contracts.BasicInboundChannel = $k1
    | .sink.contracts.IncentivizedInboundChannel = $k2
    | .source.ethereum.endpoint = $infura_endpoint_ws
    | .sink.ethereum.endpoint = $infura_endpoint_ws
    | .source.basicChannelAccounts = ($basic_parachain_account_ids | split(","))
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay.json

    # Configure ethereum relay
    jq \
        --arg k1 "$(address_for BasicOutboundChannel)" \
        --arg k2 "$(address_for IncentivizedOutboundChannel)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
    '
      .source.contracts.BasicOutboundChannel = $k1
    | .source.contracts.IncentivizedOutboundChannel = $k2
    | .source.ethereum.endpoint = $infura_endpoint_ws
    ' \
    config/ethereum-relay.json > $output_dir/ethereum-relay.json

    active_spec="mainnet"
    if [ "$eth_network" == "localhost" ]; then
       active_spec="minimal"
    fi

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
        --arg infura_endpoint_ws $infura_endpoint_ws \
        --arg k1 "$(address_for BasicOutboundChannel)" \
        --arg k2 "$(address_for IncentivizedOutboundChannel)" \
        --arg basic_eth_addresses $basic_eth_addresses \
    '
      .source.ethereum.endpoint = $infura_endpoint_ws
    | .source.contracts.BasicOutboundChannel = $k1
    | .source.contracts.IncentivizedOutboundChannel = $k2
    | .source.basicChannelAddresses = ($basic_eth_addresses | split(","))
    ' \
    config/execution-relay.json > $output_dir/execution-relay.json

    # Launch beefy relay
    (
        : > beefy-relay.log
        while :
        do
            echo "Starting beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                >>beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay
    (
        : > parachain-relay.log
        while :
        do
          echo "Starting parachain relay at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay.json" \
                --ethereum.private-key $parachain_relay_eth_key \
                >>parachain-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beacon relay
    (
        : > beacon-relay.log
        while :
        do
        echo "Starting beacon relay at $(date)"
            "${relay_bin}" run beacon \
                --config $output_dir/beacon-relay.json \
                --substrate.private-key "//BeaconRelay" \
                >>beacon-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch execution relay
    (
        : > execution-relay.log
        while :
        do
        echo "Starting execution relay at $(date)"
            "${relay_bin}" run execution \
                --config $output_dir/execution-relay.json \
                --substrate.private-key "//ExecutionRelay" \
                >>execution-relay.log 2>&1 || true
            sleep 20
        done
    ) &
}

# start_relayer
