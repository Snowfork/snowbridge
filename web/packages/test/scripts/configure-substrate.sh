#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_beacon_checkpoint() {
    # Configure beacon relay
    local electra_forked_epoch=2000000
    if [ "$is_electra" == "true" ]; then
        electra_forked_epoch=0
    fi
    jq \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json

    pushd $root_dir
    local check_point_hex=$($relay_bin generate-beacon-checkpoint --config $output_dir/beacon-relay.json)
    popd
    local transact_call="0x5200"$check_point_hex
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$transact_call" 180000000000 900000
}

wait_beacon_chain_ready() {
    local initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]; do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" |
            jq -r '.data.finalized.root' || true)
        sleep 3
    done
}

fund_accounts() {
    echo "Funding substrate accounts"
    transfer_local_balance "$bridgehub_ws_url" "//Alice" "$assethub_sovereign_account" 100000000000000
    transfer_local_balance "$bridgehub_ws_url" "//Alice" "$penpal_sovereign_account" 100000000000000
    transfer_local_balance "$bridgehub_ws_url" "//Alice" "$beacon_relayer_pub_key" 100000000000000
    transfer_local_balance "$bridgehub_ws_url" "//Alice" "$execution_relayer_assethub_pub_key" 100000000000000
    transfer_local_balance "$bridgehub_ws_url" "//Alice" "$execution_relayer_penpal_pub_key" 100000000000000
    transfer_local_balance "$assethub_ws_url" "//Alice" "$penpal_sovereign_account" 100000000000000
}

open_hrmp_channel() {
    local relay_url=$1
    local relay_chain_seed=$2
    local sender_para_id=$3
    local recipient_para_id=$4
    local max_capacity=$5
    local max_message_size=$6
    echo "  calling open_hrmp_channels:"
    echo "      relay_url: ${relay_url}"
    echo "      relay_chain_seed: ${relay_chain_seed}"
    echo "      sender_para_id: ${sender_para_id}"
    echo "      recipient_para_id: ${recipient_para_id}"
    echo "      max_capacity: ${max_capacity}"
    echo "      max_message_size: ${max_message_size}"
    echo "      params:"
    echo "--------------------------------------------------"

    call_polkadot_js_api \
        --ws "${relay_url?}" \
        --seed "${relay_chain_seed?}" \
        --sudo \
        tx.hrmp.forceOpenHrmpChannel \
        ${sender_para_id} \
        ${recipient_para_id} \
        ${max_capacity} \
        ${max_message_size}
}

open_hrmp_channels() {
    echo "Opening HRMP channels"
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1000 1002 8 512 # Assethub -> BridgeHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1002 1000 8 512 # BridgeHub -> Assethub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 2000 1002 8 512 # Penpal -> BridgeHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1002 2000 8 512 # BridgeHub -> Penpal
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1000 2000 8 512 # Penpal -> AssetHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 2000 1000 8 512 # Assethub -> Penpal
}

set_gateway() {
    echo "Setting gateway contract"
    local storage_key=$(echo $GATEWAY_STORAGE_KEY | cut -c3-)
    local gateway=$(echo $GATEWAY_PROXY_CONTRACT | cut -c3-)
    local transact_call="0x00040440"$storage_key"50"$gateway
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$transact_call"
}

config_xcm_version() {
    local call="0x1f04020109079edaa80204000000"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

register_native_eth() {
    # Registers Eth and makes it sufficient
    # https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:12144#/extrinsics/decode/0x3501020109079edaa80200ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104
    local call="0x3501020109079edaa80200ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

configure_substrate() {
    set_gateway
    fund_accounts
    open_hrmp_channels
    config_xcm_version
    wait_beacon_chain_ready
    config_beacon_checkpoint
    register_native_eth
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_substrate
    wait
fi
