#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

generate_chain_spec() {
    echo "Generating chain specification"
    "$parachain_bin" build-spec --chain "$parachain_runtime" --disable-default-bootnode > "$output_dir/spec.json"

    initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]
    do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" \
            | jq -r '.data.finalized.root')
        sleep 3
    done

    echo "Found initial finalized block: $initial_beacon_block"
    bootstrap_header=""
    slot=0
    while [ -z "$bootstrap_header" ] || [ "$bootstrap_header" == "" ] || [ "$bootstrap_header" == "null" ]
    do
        echo "Waiting for beacon to get initial bootstrap..."
        bootstrap_data=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/light_client/bootstrap/$initial_beacon_block")
        bootstrap_header=$(jq -r '.data.header' <<< "$bootstrap_data")
        slot=$(jq -r '.data.header.beacon.slot' <<< "$bootstrap_data")
        sleep 3
    done

    curl -s "$beacon_endpoint_http/eth/v1/beacon/light_client/bootstrap/$initial_beacon_block" > "$output_dir/initialBeaconSync_tmp.json"

    genesisData=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/genesis")
    validatorsRoot=$(jq -r '.data.genesis_validators_root' <<< "$genesisData")
    genesisTime=$(jq -r '.data.genesis_time' <<< "$genesisData")

    importTime="$((genesisTime + (seconds_per_slot * slot)))"

    jq \
        --arg validatorsRoot "$validatorsRoot" \
        --arg importTime "$importTime" \
        ' .validators_root = $validatorsRoot
        | .import_time = $importTime
        ' \
        "$output_dir/initialBeaconSync_tmp.json" \
        > "$output_dir/initialBeaconSync_tmp2.json"

    cat "$output_dir/initialBeaconSync_tmp2.json" | node scripts/helpers/transformInitialBeaconSync.js | sponge "$output_dir/initialBeaconSync.json"

    cat "$output_dir/spec.json" | node scripts/helpers/mutateSpec.js "$output_dir/contracts.json" "$output_dir/initialBeaconSync.json" | sponge "$output_dir/spec.json"

    echo "Generating test chain specification"
    "$test_collator_bin" build-spec --disable-default-bootnode > "$output_dir/test_spec.json"

    echo "Updating test chain specification"
    jq \
        ' .genesis.runtime.parachainInfo.parachainId = 1001
        | .para_id = 1001
        ' \
        "$output_dir/test_spec.json" | sponge "$output_dir/test_spec.json"
}

wait_start() {
    scripts/wait-for-it.sh -t 120 127.0.0.1:11144
    scripts/wait-for-it.sh -t 120 127.0.0.1:13144
}

zombienet_launch() {
    generate_chain_spec
    npx zombienet spawn config/launch-config.toml --provider=native 2>&1 &
    wait_start
}

deploy_polkadot() {
    zombienet_launch
}
