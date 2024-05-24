#!/bin/bash

set -e

source scripts/set-env.sh
source scripts/xcm-helper.sh

pushd $root_dir
check_point_hex=$($relay_bin generate-beacon-checkpoint --finalized-slot 9043968 --config /opt/config/beacon-relay.json --export-json)
popd
transact_call="0x5200"$check_point_hex
send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$transact_call" 180000000000 900000
