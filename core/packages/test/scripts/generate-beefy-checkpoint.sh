#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

generate_beefy_checkpoint()
{
    pushd "$contract_dir"
    npx ts-node ./scripts/generateBeefyCheckpoint.ts
    popd
}

if [ -z "${from_start_services:-}" ]; then
    echo "generate beefy checkpoint!"
    generate_beefy_checkpoint
    wait
fi
