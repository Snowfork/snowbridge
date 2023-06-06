#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

start_chains()
{
    echo "Starting execution node"
    geth --goerli --override.shanghai=0 --datadir="$ethereum_data_dir" --authrpc.addr="127.0.0.1" --http.addr="0.0.0.0" --http.corsdomain '*' --http --http.api eth,net,engine,admin --authrpc.jwtsecret config/jwtsecret > "$output_dir/geth.log" 2>&1 &
    echo "Waiting for geth API to be ready"
    sleep 3
    echo "Starting beacon node"
    npx lodestar beacon --dataDir="$ethereum_data_dir" --network="goerli" --rest.namespace="*" --jwt-secret="./config/jwtsecret" --checkpointSyncUrl="https://sync-goerli.beaconcha.in" > "$output_dir/lodestar.log" 2>&1 &
}

if [ -z "${from_start_services:-}" ]; then
    echo "start goerli locally!"
    trap kill_all SIGINT SIGTERM EXIT
    check_tool && cleanup && start_chains
    wait
fi
