#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

hack_lodestar() {
   echo "change variable SLOTS_PER_EPOCH from 8 to 4"
   sed -i "s/SLOTS_PER_EPOCH: 8/SLOTS_PER_EPOCH: 4/g" $core_dir/node_modules/.pnpm/@lodestar+params@1.2.2/node_modules/@lodestar/params/lib/presets/minimal/phase0.js
   echo "change variable SECONDS_PER_SLOT from 6 to 4"
   sed -i "s/SECONDS_PER_SLOT: 6/SECONDS_PER_SLOT: 4/g" $core_dir/node_modules/.pnpm/@lodestar+config@1.2.2/node_modules/@lodestar/config/lib/chainConfig/presets/minimal.js 
}

hack_lodestar