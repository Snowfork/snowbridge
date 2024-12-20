#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_relayer() {
    # Configure beacon relay
    local deneb_forked_epoch=0
    local electra_forked_epoch=256
    if [ "$eth_fast_mode" == "true" ]; then
        deneb_forked_epoch=0
    fi
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --argjson deneb_forked_epoch $deneb_forked_epoch \
        '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.spec.forkVersions.deneb = $deneb_forked_epoch
    | .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json
}

start_relayer() {
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
