#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_beacon_checkpoint()
{
    check_point_call=$($relay_bin generate-beacon-checkpoint --spec $active_spec --url $beacon_endpoint_http)
    send_governance_transact_from_relaychain $bridgehub_para_id "$check_point_call" 180000000000 900000
}

config_beacon_relayer()
{
    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --arg active_spec $active_spec \
    '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.activeSpec = $active_spec
    ' \
    config/beacon-relay.json > $output_dir/beacon-relay.json
}

wait_beacon_chain_ready()
{
    initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]
    do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" \
            | jq -r '.data.finalized.root' || true)
        sleep 3
    done
}

function configure_beacon()
{
    wait_beacon_chain_ready
    config_beacon_relayer
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    wait_beacon_chain_ready
    config_beacon_relayer
    config_beacon_checkpoint
    wait
fi
