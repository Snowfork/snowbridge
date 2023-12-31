#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

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

    npx polkadot-js-api \
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
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1000 1013 8 512 # Assethub -> BridgeHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1013 1000 8 512 # BridgeHub -> Assethub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 2000 1013 8 512 # Penpal -> BridgeHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1013 2000 8 512 # BridgeHub -> Penpal
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 1000 2000 8 512 # Penpal -> AssetHub
    open_hrmp_channel "${relaychain_ws_url}" "${relaychain_sudo_seed}" 2000 1000 8 512 # Assethub -> Penpal
}

if [ -z "${from_start_services:-}" ]; then
    echo "open hrmp channels only!"
    open_hrmp_channels
    wait
fi
