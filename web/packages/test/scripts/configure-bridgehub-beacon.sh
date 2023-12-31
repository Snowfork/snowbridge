#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_beacon_checkpoint() {
    pushd $root_dir
    local check_point_bytes=$($relay_bin generate-beacon-checkpoint --spec $active_spec --url $beacon_endpoint_http)
    popd
    local check_point_call="0x5200"$check_point_bytes
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID $check_point_call 360000000000 1800000
}

wait_beacon_chain_ready() {
    local initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]; do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" |
            jq -r '.data.finalized.root' || true)
        sleep 3
    done
}

configure_bridgehub_beacon() {
    wait_beacon_chain_ready
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_bridgehub_beacon
    wait
fi
