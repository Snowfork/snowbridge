#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_pallet_owner()
{
    local owner=$(echo $bridgehub_pallets_owner | cut -c3-)
    local option="01"

    # config owner of inbound queue
    local pallet="30"
    local callindex="03"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$payload"

    # config owner of outbound queue
    local pallet="31"
    local callindex="00"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$payload"

    # config owner of beacon client owner
    local pallet="32"
    local callindex="03"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$payload"
}


configure_bridgehub()
{
    config_pallet_owner
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_bridgehub
    wait
fi
