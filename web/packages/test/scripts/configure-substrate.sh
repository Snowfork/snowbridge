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

configure_others() {
    echo "Configure others"
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
    # Mint Ether
    local call="0x3506020109079edaa802001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
}

configure_substrate() {
    configure_bh
    configure_ah
    configure_others
    # The beacon checkpoint is a large call and should be separated.
    wait_beacon_chain_ready
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    configure_substrate
    wait
fi
