#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

send_governance_transact_from_relaychain() {
    local para_id=$1
    local hex_encoded_data=$2
    local require_weight_at_most_ref_time=${3:-2000000000}
    local require_weight_at_most_proof_size=${4:-12000}
    echo "  calling send_governance_transact:"
    echo "      relay_url: ${relaychain_ws_url}"
    echo "      relay_chain_seed: ${relaychain_sudo_seed}"
    echo "      para_id: ${para_id}"
    echo "      require_weight_at_most_ref_time: ${require_weight_at_most_ref_time}"
    echo "      require_weight_at_most_proof_size: ${require_weight_at_most_proof_size}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg para_id "$para_id" \
                    '{ "v3": { "parents": 0, "interior": { "x1": { "parachain": $para_id } } } }')

    local message=$(jq --null-input \
                       --arg hex_encoded_data "$hex_encoded_data" \
                       --arg require_weight_at_most_ref_time "$require_weight_at_most_ref_time" \
                       --arg require_weight_at_most_proof_size "$require_weight_at_most_proof_size" \
                       '
                       {
                          "v3": [
                                  {
                                    "unpaidexecution": {
                                        "weight_limit": "unlimited"
                                    }
                                  },
                                  {
                                    "transact": {
                                      "origin_kind": "superuser",
                                      "require_weight_at_most": {
                                        "ref_time": $require_weight_at_most_ref_time,
                                        "proof_size": $require_weight_at_most_proof_size,
                                      },
                                      "call": {
                                        "encoded": $hex_encoded_data
                                      }
                                    }
                                  }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    call_polkadot_js_api \
        --ws "${relaychain_ws_url?}" \
        --seed "${relaychain_sudo_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}

transfer_balance() {
    local runtime_para_endpoint=$1
    local seed=$2
    local para_id=$3
    local amount=$4
    local target_account=$5

    local dest=$(jq --null-input \
                    --arg para_id "$para_id" \
                    '{ "v3": { "parents": 0, "interior": { "x1": { "parachain": $para_id } } } }')
    local beneficiary=$(jq --null-input \
                    --arg target_account "$target_account" \
                    '{ "v3": { "parents": 0, "interior": { "x1": { "accountid32": { "id": $target_account } } } } }')
    local assets=$(jq --null-input \
                    --arg amount "$amount" \
        '
        {
            "V3": [
                {
                    "id": {
                        "Concrete": {
                            "parents": 0,
                            "interior": "Here"
                        }
                    },
                    "fun": {
                        "Fungible": $amount
                    }
                }
            ]
        }
        '
    )
    local asset_fee_item=0

    echo "  calling transfer_balance:"
    echo "      target_account: ${target_account}"
    echo "      dest: ${dest}"
    echo "      beneficiary: ${beneficiary}"
    echo "      assets: ${assets}"
    echo "      asset_fee_item: ${asset_fee_item}"
    echo "--------------------------------------------------"

    call_polkadot_js_api \
        --ws "${runtime_para_endpoint}" \
        --seed "${seed?}" \
        tx.xcmPallet.transferAssets \
            "${dest}" \
            "${beneficiary}" \
            "${assets}" \
            "${asset_fee_item}" \
            "Unlimited"
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

function call_polkadot_js_api() {
    # --noWait: without that argument `polkadot-js-api` waits until transaction is included into the block.
    #           With it, it just submits it to the tx pool and exits.
    # --nonce -1: means to compute transaction nonce using `system_accountNextIndex` RPC, which includes all
    #             transaction that are in the tx pool.
    # TODO: add back nowait and nonce: npx polkadot-js-api --noWait --nonce -1 "$@" || true
    npx polkadot-js-api "$@" || true
}
