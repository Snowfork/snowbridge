#!/usr/bin/env sh

if [[ "$(uname)" == "Darwin" && -z "${IN_NIX_SHELL:-}" ]]; then
    alias sed='gsed'
fi

echo "Update submodules"
(cd .. && (git submodule update --init --recursive||true))

echo "Install husky hook"
(cd .. && ./core/node_modules/.bin/husky install)

echo "Initialize foundry libraries"
(cd packages/contracts && (forge install||true))

echo "Hack lodestar for faster slot time"
. packages/test/scripts/set-env.sh
preset_minimal_config_file="./node_modules/.pnpm/@lodestar+config@$lodestar_version/node_modules/@lodestar/config/lib/chainConfig/presets/minimal.js"
sed -i "s/SECONDS_PER_SLOT: 6/SECONDS_PER_SLOT: 2/g" $preset_minimal_config_file
