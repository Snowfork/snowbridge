#!/bin/bash

set -e

source scripts/set-env.sh
source scripts/xcm-helper.sh

pushd $root_dir
check_point_hex=$($relay_bin generate-beacon-checkpoint --config /opt/config/beacon-relay.json --export-json)
popd
transact_call="0x5200"$check_point_hex
echo $transact_call
