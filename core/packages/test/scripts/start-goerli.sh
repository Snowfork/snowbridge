#!/usr/bin/env bash
set -eu

source scripts/set-env.sh


start_chains()
{
    geth --goerli --override.shanghai=0 --datadir="$ethereum_data_dir" --authrpc.addr="127.0.0.1" --http.addr="0.0.0.0" --http.corsdomain '*' --http --http.api eth,net,engine,admin --authrpc.jwtsecret config/jwtsecret 2>&1 &
    npx lodestar beacon --dataDir="$ethereum_data_dir" --network="goerli" --rest.namespace="*" --jwt-secret="./config/jwtsecret" --checkpointSyncUrl="https://sync-goerli.beaconcha.in" 2>&1 &
}

if [ -z "${from_start_services:-}" ]; then
    echo "start goerli locally!"
    trap kill_all SIGINT SIGTERM EXIT
    start_chains
    wait
fi
