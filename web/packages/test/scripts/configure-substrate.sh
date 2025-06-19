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
    # Mint Ether to Alice
    local call="0x3506020109079edaa80200d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d1300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
    # Mint Ether to Ferdie
    local call="0x3506020109079edaa802001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
    # Create Pool for Ether<->Wnd and add liquidity
    local call="0x38000100020109079edaa802"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    local call="0x38010100020109079edaa8020000e941cc6b01000000000000000000000064a7b3b6e00d000000000000000001000000000000000000000000000000010000000000000000000000000000001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    # register Wnd on BH
    local call="0x24010501000c776e640c776e640c020109079edaa8020002286bee"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # Register Roc
    register_roc
}

register_roc() {
    # Register Roc on AH
    local call="0x28020c1f04020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e050000003501020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d01043513020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e0c526f630c526f630c00"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # Register Roc on BH
    local call="0x240105020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e0c726f630c726f630c020109079edaa8020002286bee"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # Mint Roc to Ferdie
    local call="0x3506020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
}


configure_substrate() {
    configure_from_test_helper
    sleep 3
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
