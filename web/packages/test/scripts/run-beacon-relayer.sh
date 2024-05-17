#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

# Launch beacon relay
(
    : >"$output_dir"/beacon-relay.log
    while :; do
        echo "Starting beacon relay at $(date)"
        "${relay_bin}" run beacon \
            --config /opt/config/beacon-relay.json \
            --substrate.private-key "//BeaconRelay" \
            >>"$output_dir"/beacon-relay.log 2>&1 || true
        sleep 20
    done
) &



wait
