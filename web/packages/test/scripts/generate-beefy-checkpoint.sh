#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

generate_beefy_checkpoint()
{
    pushd "$test_helpers_dir"
    pnpm generateBeefyCheckpoint
    if [ "$snowbridge_v1" = true ]; then
        cp "$contract_dir/beefy-state.json" "$v1_contract_dir"
    fi
    popd
}

if [ -z "${from_start_services:-}" ]; then
    echo "generate beefy checkpoint!"
    generate_beefy_checkpoint
    wait
fi
