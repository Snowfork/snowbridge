#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_beacon_checkpoint() {
    # Configure beacon relay
    local electra_forked_epoch=0
    jq \
        --argjson electra_forked_epoch $electra_forked_epoch \
        '
      .source.beacon.spec.forkVersions.electra = $electra_forked_epoch
    ' \
        config/beacon-relay.json >$output_dir/beacon-relay.json

    pushd $root_dir
    local check_point_hex=$($relayer_v2 generate-beacon-checkpoint --config $output_dir/beacon-relay.json)
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

# A batch call to create all HRMP channels and provide some initial funding
configure_from_test_helper() {
    echo "Configure from test helper"
    pushd ../test-helpers
    pnpm configureE2E
    popd
}

set_gateway() {
    echo "Setting gateway contract"
    local storage_key=$(echo $GATEWAY_STORAGE_KEY | cut -c3-)
    local gateway=$(echo $GATEWAY_PROXY_CONTRACT | cut -c3-)
    local transact_call="0x00040440"$storage_key"50"$gateway
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$transact_call"
}


configure_bh() {
   set_gateway
}

configure_ah() {
    # Create Ether
    local call="0x28020c1f04020109079edaa802040000003501020109079edaa80200ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d01043513020109079edaa8021445746865721445746865721200"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"

    # register Wnd on BH
    local call="0x24010501000c776e640c776e640c020109079edaa8020002286bee"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

configure_substrate() {
    configure_from_test_helper
    configure_bh
    configure_ah
    wait_beacon_chain_ready
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_substrate
    wait
fi
