#!/usr/bin/env bash
set -eu
# export BEACON_HTTP_ENDPOINT=https://lodestar-goerli.chainsafe.io
source scripts/set-env.sh

# finality_update
curl -s "$beacon_endpoint_http/eth/v1/beacon/light_client/finality_update" | jq -r "." > testdata/finality_update.json
finalized_block_number=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/light_client/finality_update" | jq -r ".data.finalized_header.beacon.slot")
echo "finalized_block_number is: $finalized_block_number"

# get block_root by block_number
finalized_block_root=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/blocks/$finalized_block_number/root" | jq -r ".data.root")
echo "finalized_block_root is:" $finalized_block_root

# get beacon header by block_root
beacon_header=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/headers/$finalized_block_root")
echo "beacon header is:" $beacon_header

# get validators root from genesis
validatorsRoot=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/genesis" | jq -r ".data.genesis_validators_root")
echo "validators root is:" $validatorsRoot

# get checkpoint
checkpoint_root=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" | jq -r ".data.finalized.root")
echo "checkpoint_root is:" $checkpoint_root
curl -s "$beacon_endpoint_http/eth/v1/beacon/light_client/bootstrap/$checkpoint_root" | jq -r "." > testdata/beacon_check_point.json

# get beacon block
echo "fetch beacon block..."
curl -s "$beacon_endpoint_http/eth/v2/beacon/blocks/$finalized_block_root" | jq -r "." > testdata/beacon_block.json

if [[ $beacon_endpoint_http == "http://127.0.0.1:9596" ]];then
    echo "fetch beacon state..."
    curl -s "$beacon_endpoint_http/eth/v2/debug/beacon/states/$finalized_block_number" | jq -r "." > testdata/beacon_state.json
fi
