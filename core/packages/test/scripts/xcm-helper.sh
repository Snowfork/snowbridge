#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

statemine_trap_message() {
    echo Sending trap message.

    echo "  calling send_xcm_trap_from_statemine:"
    echo "      seed: ${statemine_seed}"
    echo "      statemine_ws_url: ${statemine_ws_url}"
    echo "      statemine_para_id: ${statemine_para_id}"
    echo "      bridgehub_ws_url: ${bridgehub_ws_url}"
    echo "      bridgehub_para_id: ${bridgehub_para_id}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg bridgehub_para_id "$bridgehub_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridgehub_para_id } } } }')

    local message=$(jq --null-input \
                       '
                       {
                          "V3": [
                            {
                              "UnpaidExecution": {
                                "weight_limit": "Unlimited"
                              }
                            },
                            {
                              "ExportMessage": {
                                "network": {
                                    "Ethereum": { chainId: 1 }
                                },
                                "destination": {
                                  "Here": "Null"
                                },
                                "xcm": [
                                  {
                                    "Trap": 12345
                                  }
                                ]
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

    npx polkadot-js-api \
        --ws "${statemine_ws_url?}" \
        --seed "${statemine_seed?}" \
        tx.polkadotXcm.send \
            "${dest}" \
            "${message}"
}

bridgehub_trap_message() {
    echo Sending trap message.

    echo "  calling send_xcm_trap_from_bridgehub:"
    echo "      seed: ${bridgehub_seed}"
    echo "      statemine_ws_url: ${statemine_ws_url}"
    echo "      statemin_para_id: ${statemine_para_id}"
    echo "      bridgehub_ws_url: ${bridgehub_ws_url}"
    echo "      bridgehub_para_id: ${bridgehub_para_id}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg statemine_para_id "$statemine_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $statemine_para_id } } } }')

    local weight='{ "refTime": 900000000, "proofSize": 10000 }'

    local message=$(jq --null-input \
                       '
                       {
                          "V3": [
                            {
                              "UnpaidExecution": {
                                "weight_limit": "Unlimited"
                              }
                            },
                            {
                              "ExportMessage": {
                                "network": {
                                    "Ethereum": { chainId: 1 }
                                },
                                "destination": {
                                  "Here": "Null"
                                },
                                "xcm": [
                                  {
                                    "Trap": 12345
                                  }
                                ]
                              }
                            }
                          ]
                        }
                        ')

    echo ""
    echo "          dest:"
    echo "${dest}"
    echo ""
    echo "          weight:"
    echo "${weight}"
    echo ""
    echo "          message:"
    echo "${message}"
    echo ""
    echo "--------------------------------------------------"

    npx polkadot-js-api \
        --ws "${bridgehub_ws_url?}" \
        --seed "${bridgehub_seed?}" \
        tx.polkadotXcm.execute \
            "${message}" \
            "${weight}"
}

send_governance_transact_from_relaychain() {
    local para_id=$1
    local hex_encoded_data=$2
    local require_weight_at_most_ref_time=$3
    local require_weight_at_most_proof_size=$4
    echo "  calling send_governance_transact:"
    echo "      relay_url: ${relaychain_ws_url}"
    echo "      relay_chain_seed: ${relaychain_sudo_seed}"
    echo "      para_id: ${para_id}"
    echo "      hex_encoded_data: ${hex_encoded_data}"
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

    npx polkadot-js-api \
        --ws "${relaychain_ws_url?}" \
        --seed "${relaychain_sudo_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}
