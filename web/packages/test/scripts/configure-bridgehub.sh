#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

fund_accounts() {
    echo "Funding substrate accounts"
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $assethub_sovereign_account
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $penpal_sovereign_account
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $beacon_relayer_pub_key
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $execution_relayer_pub_key
}

set_gateway() {
    echo "Setting gateway contract"
    local storage_key=$(echo $GATEWAY_STORAGE_KEY | cut -c3-)
    local gateway=$(echo $GATEWAY_PROXY_CONTRACT | cut -c3-)
    local transact_call="0x00040440"$storage_key"50"$gateway
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$transact_call"
}

configure_bridgehub() {
    set_gateway
    fund_accounts
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_bridgehub
    wait
fi
