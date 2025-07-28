#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

register_weth_on_ah() {
    # register weth
    local call='0x3501020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d00ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d0104'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # set metadata
    local call='0x3513020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
    # Mint weth to Penpal Sovereign on AH
    local call="0x3506020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d007369626cd00700000000000000000000000000000000000000000000000000001300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
    # Mint weth to Ferdie
   # local call='0x3506020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229'
  #  send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
}

register_wnd_on_ethereum() {
    local call="0x24010501000c776e640c776e640c020109079edaa8020002286bee"
    send_governance_transact_from_relaychain $ASSET_HUB_PARAID "$call"
}

register_weth_on_penpal() {
    # register weth
    local call='0x3301020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0104'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # set weth meta data
    local call='0x3313020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d105745544810574554481200'
    send_governance_transact_from_relaychain $PENPAL_PARAID "$call"
    # Mint weth to ferdie
    local call='0x3306020209079edaa8020300b8ea8cb425d85536b158d661da1ef0895bb92f1d001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229'
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
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
    # Mint Ether to Penpal Sovereign on AH
    local call="0x3506020109079edaa802007369626cd00700000000000000000000000000000000000000000000000000001300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
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
    # mint Pal-2 to Ferdie on AH
   # local call='0x3506010300411f04320508001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c0f0080c6a47e8d03'
  #  send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    # mint Pal-2 to Ferdie on Penpal
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

mint_wnd_as_fee() {
    # Mint Wnd
    local call='0x33060100001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229'
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
}

register_roc_on_ah() {
  #  # Set admin of Roc to Alice
  #  local call="0x3515020109006408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d040100"
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

configure_bridge() {
    # fund on penpal
    transfer_local_balance "$penpal_ws_url" "//Alice" "$assethub_sovereign_account" 1000000000000
    transfer_local_balance "$penpal_ws_url" "//Alice" "$checking_account" 1000000000000
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

add_liquidity_on_ah() {
    # Mint Ether to Alice
    local call="0x3506020109079edaa80200d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d1300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
    # Mint Ether to Ferdie
    local call="0x3506020109079edaa802001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229"
    send_transact_through_bridge_from_relaychain $ASSET_HUB_PARAID "$call"
    # Create Pool for Ether<->Wnd and add liquidity
    local call="0x38000100020109079edaa802"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
    local call="0x38010100020109079edaa8020080c6a47e8d0300000000000000000000008d49fd1a0700000000000000000001000000000000000000000000000000010000000000000000000000000000001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"
    send_transact_through_user_origin_from_relaychain $ASSET_HUB_PARAID "$sudo_pubkey" "$call"
}

add_liquidity_on_penpal() {
    # Mint Ether to Alice
    local call="0x3306020109079edaa80200d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d1300002cf61a24a229"
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
    # Mint Ether to Ferdie
    local call="0x3306020109079edaa802001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c1300002cf61a24a229"
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
    # Create Pool for Ether<->pal and add liquidity
    local call="0x35000100020109079edaa802"
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
    local call="0x35010100020109079edaa8020010a5d4e8000000000000000000000000c817a804000000000000000000000001000000000000000000000000000000010000000000000000000000000000001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"
    send_transact_through_user_origin_from_relaychain $PENPAL_PARAID "$sudo_pubkey" "$call"
}

configure_on_penpal() {
    configure_bridge
    register_ether
    register_weth_on_penpal
    register_pal
    mint_wnd_as_fee
    add_liquidity_on_penpal
}

configure_on_ah() {
    register_weth_on_ah
    register_wnd_on_ethereum
    register_roc_on_ah
    add_liquidity_on_ah
}

function configure_all() {
    configure_on_ah
    configure_on_penpal
}

if [ -z "${from_start_services:-}" ]; then
    echo "config Penpal for tests"
    configure_all
    wait
fi
