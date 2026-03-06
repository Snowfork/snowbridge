#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_beacon_state_service() {
    local electra_forked_epoch=0
    local fulu_forked_epoch=0

    # Configure beacon state service
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --argjson electra_forked_epoch $electra_forked_epoch \
        --argjson fulu_forked_epoch $fulu_forked_epoch \
        --arg datastore_location "$output_dir/beacon-state-service-data" \
        '
      .beacon.endpoint = $beacon_endpoint_http
    | .beacon.spec.forkVersions.electra = $electra_forked_epoch
    | .beacon.spec.forkVersions.fulu = $fulu_forked_epoch
    | .beacon.datastore.location = $datastore_location
    ' \
        config/beacon-state-service.json >$output_dir/beacon-state-service.json
}

start_beacon_state_service() {
    # Launch beacon state service (before other relayers since may use it)
    (
        : >"$output_dir"/beacon-state-service.log
        while :; do
            echo "Starting beacon state service at $(date)"
            "${relayer}" run beacon-state-service \
                --config "$output_dir/beacon-state-service.json" \
                >>"$output_dir"/beacon-state-service.log 2>&1 || true
            sleep 20
        done
    ) &
}

build_relayer() {
    echo "Building relayer"
    mage -d "$relay_dir" build
    rm -rf $relayer
    cp $relay_bin $relayer
}

deploy_beacon_state_service() {
    check_tool && build_relayer && config_beacon_state_service && start_beacon_state_service
}

if [ -z "${from_start_services:-}" ]; then
    echo "start relayers only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_beacon_state_service
    wait
fi
