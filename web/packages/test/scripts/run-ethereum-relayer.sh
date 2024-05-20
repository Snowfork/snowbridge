#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

# Launch ethereum assethub relay
(
    : >$output_dir/execution-relay-asset-hub.log
    while :; do
        echo "Starting execution relay (asset-hub) at $(date)"
        "${relay_bin}" run execution \
            --config /opt/config/execution-relay.json \
            --substrate.private-key "//ExecutionRelayAssetHub" \
            >>"$output_dir"/execution-relay-asset-hub.log 2>&1 || true
        sleep 20
    done
) &



wait
