#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

fund_sender_sovereign() {
    # forceSetBalance($sender_sovereign_account, 1000000000000)
    local transact_call="0x0a0800"$sender_sovereign_account"070010a5d4e8"
    send_governance_transact_from_relaychain $PENPAL_PARAID "$transact_call"
}

configure_penpal() {
    fund_sender_sovereign
}

if [ -z "${from_start_services:-}" ]; then
    echo "config penpal only!"
    configure_penpal
    wait
fi
