#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

hack_lodestar() {
   preset_minimal_phase0_file="$core_dir/node_modules/.pnpm/@lodestar+params@$lodestar_version/node_modules/@lodestar/params/lib/presets/minimal.js"
   if [[ "$OSTYPE" =~ ^darwin ]]
   then
      sed_cmd=gsed
   else
      sed_cmd=sed
   fi
   echo "change variable SLOTS_PER_EPOCH from 8 to 4."
   $sed_cmd -i "s/SLOTS_PER_EPOCH: 8/SLOTS_PER_EPOCH: 4/g" $preset_minimal_phase0_file
}

hack_lodestar
