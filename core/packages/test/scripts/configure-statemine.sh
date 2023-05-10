#!/usr/bin/env sh

set -eu

send_governance_transact() {
    local relay_url=$1
    local relay_chain_seed=$2
    local para_id=$3
    local hex_encoded_data=$4
    local require_weight_at_most_ref_time=$5
    local require_weight_at_most_proof_size=$6
    echo "  calling send_governance_transact:"
    echo "      relay_url: ${relay_url}"
    echo "      relay_chain_seed: ${relay_chain_seed}"
    echo "      para_id: ${para_id}"
    echo "      hex_encoded_data: ${hex_encoded_data}"
    echo "      require_weight_at_most_ref_time: ${require_weight_at_most_ref_time}"
    echo "      require_weight_at_most_proof_size: ${require_weight_at_most_proof_size}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg para_id "$para_id" \
                    '{ "v3": { "parents": 0, "interior": { "x1": { "parachain": $para_id } } } }')

    local message=$(jq --null-input \
                       --argjson hex_encoded_data $hex_encoded_data \
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
        --ws "${relay_url?}" \
        --seed "${relay_chain_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}

add_exporter_config() {
    local relay_url=$1
    local relay_chain_seed=$2
    local statemine_para_id=$3
    local statemine_para_endpoint=$4
    local bridge_hub_para_id=$5
    local ethereum_chain_id=$6

    echo "  calling add_exporter_config:"
    echo "      relay_url: ${relay_url}"
    echo "      relay_chain_seed: ${relay_chain_seed}"
    echo "      statemine_para_id: ${statemine_para_id}"
    echo "      runtime_para_endpoint: ${statemine_para_endpoint}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      ethereum_chain_id: ${ethereum_chain_id}"
    echo "      params:"

    local bridged_network=$(jq --null-input \
                               --arg ethereum_chain_id "$ethereum_chain_id" \
      '
        { 
          "Ethereum": { 
            "chainId": $ethereum_chain_id 
          } 
        }
      '
    )

    # Generate data for Transact (add_exporter_config)
    local bridge_config=$(jq --null-input \
                             --arg bridge_hub_para_id "$bridge_hub_para_id" \
                             --arg bridged_network "$bridged_network" \
        '
            {
                "bridgeLocation": {
                    "parents": 1,
                    "interior": {
                        "X1": { "Parachain": $bridge_hub_para_id }
                    }
                },
                "allowedTargetLocation": {
                    "parents": 2,
                    "interior": {
                        "X1": {
                            "GlobalConsensus": $bridged_network | fromjson,
                        }
                    }
                }
            }
        '
    )
    local tmp_output_file=$(mktemp)
    node scripts/helpers/generateBridgeConfig.js \
      "add-exporter-config" \
      "$statemine_para_endpoint" \
      "$tmp_output_file" \
      "$bridged_network" \
      "$bridge_config"
    local hex_encoded_data=$(cat $tmp_output_file)
    rm $tmp_output_file

    send_governance_transact "${relay_url}" "${relay_chain_seed}" "${statemine_para_id}" "${hex_encoded_data}" 200000000 12000
}

if [ "$#" -eq 0 ]; then
    cat <<EOF
usage:
    $0 messages

    messages:
        add-exporter-config - adds bridge config for ethereum.
EOF
    exit 1
fi

while [ "$#" -gt 0 ]; do
  case "$1" in
    add-exporter-config)
      shift
      add_exporter_config $1 $2 $3 $4 $5 $6
      shift 6
      ;;
    *)
      echo "Unknown message: $1"
      exit 1
      ;;
  esac
done

exit 0
