#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

config_ah() {
   register_weth_on_ah
   register_pal_on_ah
}

register_weth_on_ah() {
    # register token
    local call='0x3501020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d00ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # set metadata
    local call='0x3513020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

register_pal_on_ah() {
    # register token
    local call='0x3501010300411f0432050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # set metadata
    local call="0x3513010300411f043205081450616c20321450616c2d320c00"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

config_bh() {
    register_wnd_on_bh
    register_pal_on_bh
}

register_wnd_on_bh() {
    # register WND
    local call='0x530a0401000c776e640c776e640c'
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$call"
}

register_pal_on_bh() {
    # register PAL-2
    local call='0x530a04010300411f043205081470616c2d321470616c2d320c'
    send_governance_transact_from_relaychain $BRIDGE_HUB_PARAID "$call"
}

config_penpal() {
    fund_on_penpal
    config_bridge_on_penpal
    register_weth_on_penpal
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

register_weth_on_penpal() {
    # register weth
    local call='0x3301020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d00ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # set weth meta data
    local call='0x3313020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
}


if [ -z "${from_start_services:-}" ]; then
    echo "config others for PNA tests"
    config_ah
    config_bh
    config_penpal
    wait
fi
