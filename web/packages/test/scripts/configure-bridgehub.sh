#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_beacon_checkpoint()
{
    pushd $root_dir
    local check_point_call=$($relay_bin generate-beacon-checkpoint --spec $active_spec --url $beacon_endpoint_http)
    popd
    send_governance_transact_from_relaychain $bridgehub_para_id "$check_point_call" 180000000000 900000
}

config_inbound_queue()
{
    local pallet="30"
    local callindex="01"
    local payload="0x$pallet$callindex$(address_for GatewayProxy | tr "[:upper:]" "[:lower:]" | cut -c3-)"

    send_governance_transact_from_relaychain $bridgehub_para_id "$payload"
}

config_pallet_owner()
{
    local owner=$(echo $bridgehub_pallets_owner | cut -c3-)
    local option="01"

    # config owner of inbound queue
    local pallet="30"
    local callindex="03"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $bridgehub_para_id "$payload"

    # config owner of outbound queue
    local pallet="31"
    local callindex="00"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $bridgehub_para_id "$payload"

    # config owner of beacon client owner
    local pallet="32"
    local callindex="03"
    local payload="0x$pallet$callindex$option$owner"
    send_governance_transact_from_relaychain $bridgehub_para_id "$payload"
}

wait_beacon_chain_ready()
{
    local initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]
    do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" \
            | jq -r '.data.finalized.root' || true)
        sleep 3
    done
}

fund_accounts()
{
    echo "Funding substrate accounts"
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $statemine_sovereign_account
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $beacon_relayer_pub_key
    transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $execution_relayer_pub_key
    transfer_balance $relaychain_ws_url "//Charlie" 1000 1000000000000000 $gateway_contract_sovereign_account
}

configure_bridgehub()
{
    fund_accounts
    config_inbound_queue
    config_pallet_owner
    wait_beacon_chain_ready
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_bridgehub
    wait
fi
