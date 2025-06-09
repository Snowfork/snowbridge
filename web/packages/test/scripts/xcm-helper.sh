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
                    '{ "v4": { "parents": 0, "interior": { "x1": [{ "parachain": $para_id }] } } }')

    local message=$(jq --null-input \
                       --arg hex_encoded_data "$hex_encoded_data" \
                       --arg require_weight_at_most_ref_time "$require_weight_at_most_ref_time" \
                       --arg require_weight_at_most_proof_size "$require_weight_at_most_proof_size" \
                       '
                       {
                          "v4": [
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

function call_polkadot_js_api() {
    # --noWait: without that argument `polkadot-js-api` waits until transaction is included into the block.
    #           With it, it just submits it to the tx pool and exits.
    # --nonce -1: means to compute transaction nonce using `system_accountNextIndex` RPC, which includes all
    #             transaction that are in the tx pool.
    # TODO: add back nowait and nonce: npx polkadot-js-api --noWait --nonce -1 "$@" || true
    npx polkadot-js-api "$@" || true
}


send_transact_through_bridge_from_relaychain() {
    local para_id=$1
    local hex_encoded_data=$2
    local require_weight_at_most_ref_time=${3:-2000000000}
    local require_weight_at_most_proof_size=${4:-900000}
    echo "  calling send_transact_as_bridge_from_relaychain:"
    echo "      relay_url: ${relaychain_ws_url}"
    echo "      relay_chain_seed: ${relaychain_sudo_seed}"
    echo "      para_id: ${para_id}"
    echo "      require_weight_at_most_ref_time: ${require_weight_at_most_ref_time}"
    echo "      require_weight_at_most_proof_size: ${require_weight_at_most_proof_size}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg para_id "$para_id" \
                    '{ "v4": { "parents": 0, "interior": { "x1": [{ "parachain": $para_id }] } } }')

    local message=$(jq --null-input \
                       --arg hex_encoded_data "$hex_encoded_data" \
                       --arg require_weight_at_most_ref_time "$require_weight_at_most_ref_time" \
                       --arg require_weight_at_most_proof_size "$require_weight_at_most_proof_size" \
                       '
                       {
                          "v4": [
                                  {
                                    "unpaidexecution": {
                                        "weight_limit": "unlimited"
                                    }
                                  },
                                  {
                                    "descendOrigin": {
                                        "x2": [{"parachain": 1002},{"palletInstance": 91}]
                                    }
                                  },
                                  {
                                    "universalOrigin": {
                                        "globalConsensus": {"ethereum": {"chainId": 11155111}}
                                    }
                                  },
                                  {
                                    "transact": {
                                      "origin_kind": "sovereignaccount",
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


send_transact_through_user_origin_from_relaychain() {
    local para_id=$1
    local account_id=$2
    local hex_encoded_data=$3
    local require_weight_at_most_ref_time=${4:-2000000000}
    local require_weight_at_most_proof_size=${5:-900000}
    echo "  calling send_transact_through_user_origin_from_relaychain:"
    echo "      relay_url: ${relaychain_ws_url}"
    echo "      relay_chain_seed: ${relaychain_sudo_seed}"
    echo "      para_id: ${para_id}"
    echo "      account_id: ${account_id}"
    echo "      require_weight_at_most_ref_time: ${require_weight_at_most_ref_time}"
    echo "      require_weight_at_most_proof_size: ${require_weight_at_most_proof_size}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg para_id "$para_id" \
                    '{ "v4": { "parents": 0, "interior": { "x1": [{ "parachain": $para_id }] } } }')

    local message=$(jq --null-input \
                       --arg para_id "$para_id" \
                       --arg account_id "$account_id" \
                       --arg hex_encoded_data "$hex_encoded_data" \
                       --arg require_weight_at_most_ref_time "$require_weight_at_most_ref_time" \
                       --arg require_weight_at_most_proof_size "$require_weight_at_most_proof_size" \
                       '
                       {
                          "v4": [
                                  {
                                    "unpaidexecution": {
                                        "weight_limit": "unlimited"
                                    }
                                  },
                                  {
                                    "descendOrigin": {
                                        "x1": [{"accountId32": {"network": null, "id":$account_id}}]
                                    }
                                  },
                                  {
                                    "aliasOrigin": {
                                      "parents": 0,
                                      "interior": { "x1": [{"accountId32": {"network": null, "id":$account_id}}]}
                                    }
                                  },
                                  {
                                    "transact": {
                                      "origin_kind": "sovereignaccount",
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
