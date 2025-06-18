#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh


register_weth() {
    register_weth_on_ah
    register_weth_on_penpal
}

register_weth_on_ah() {
    # register token
    local call='0x3501020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d00ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # set metadata
    local call='0x3513020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

register_weth_on_penpal() {
    # register weth
    local call='0x3301020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c0104'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # set weth meta data
    local call='0x3313020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
}

register_ether() {
    # register ether
    local call='0x3301020109079edaa80200d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0104'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # set meta data
    local call='0x3313020109079edaa8021445746865721445746865721200'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # Mint Ether
    local call='0x3306020109079edaa802001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229'
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
    # Mint Wnd
    local call='0x33060100001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229'
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
}


register_pal() {
    # register pal-2
    local call='0x3501010300411f0432050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # set pal-2 metadata
    local call="0x3513010300411f043205081470616c2d321470616c2d320c00"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # register pal-2 on bh
    local call='0x240105010300411f043205081470616c2d321470616c2d320c020109079edaa8020002286bee'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # mint Pal-2 to Ferdie
    local call='0x320608001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c0b0030ef7dba02'
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"

    # register native pal
    local call='0x3501010100411f00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # mint pal
    local call='0x3506010100411f00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0f0080c6a47e8d03'
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    # Create Pool for Pal<->Wnd and add liquidity
    local call="0x38000100010100411f"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    local call="0x38010100010100411f00a0724e18090000000000000000000000a0724e18090000000000000000000001000000000000000000000000000000010000000000000000000000000000001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
}


config_penpal() {
    fund_on_penpal
    config_bridge_on_penpal
}

fund_on_penpal() {
    transfer_local_balance "$penpal_ws_url" "//Alice" "$assethub_sovereign_account" 1000000000000
    transfer_local_balance "$penpal_ws_url" "//Alice" "$checking_account" 1000000000000
}

config_bridge_on_penpal() {
    # config bridge sovereign as reserve
    local call='0x00040440770800eb78be69c327d8334d0927607220020109079edaa802'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
}

function transfer_local_balance() {
    local runtime_para_endpoint=$1
    local seed=$2
    local target_account=$3
    local amount=$4
    echo "  calling transfer_balance:"
    echo "      runtime_para_endpoint: ${runtime_para_endpoint}"
    echo "      seed: ${seed}"
    echo "      target_account: ${target_account}"
    echo "      amount: ${amount}"
    echo "--------------------------------------------------"

    call_polkadot_js_api \
        --ws "${runtime_para_endpoint}" \
        --seed "${seed?}" \
        tx.balances.transferAllowDeath \
            "${target_account}" \
            "${amount}"
}

function configure_penpal() {
    config_penpal
    register_ether
    register_weth
    register_pal
}


if [ -z "${from_start_services:-}" ]; then
    echo "config Penpal for tests"
    configure_penpal
    wait
fi
