#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

fund_bridgehub_sovereign() {
    local call="0x0a08007369626cf5030000000000000000000000000000000000000000000000000000070010a5d4e8"
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
}

configure_penpal() {
    fund_bridgehub_sovereign
}

if [ -z "${from_start_services:-}" ]; then
    echo "config penpal only!"
    configure_penpal
    wait
fi
