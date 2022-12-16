#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh

check_tool
mkdir -p "$output_bin_dir"
rebuild_relaychain
build_parachain
build_relayer

