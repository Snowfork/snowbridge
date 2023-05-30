#!/usr/bin/env sh
set -eu

source scripts/xcm-helper.sh

generate_hex_encoded_call_data() {
    local type=$1
    local endpoint=$2
    local output=$3
    shift
    shift
    shift
    echo "Input params: $@"

    node scripts/helpers/generateHexEncodedCallData.js "$type" "$endpoint" "$output" "$@"
    local retVal=$?

    if [ $type != "check" ]; then
        local hex_encoded_data=$(cat $output)
        echo "Generated hex-encoded bytes to file '$output': $hex_encoded_data"
    fi

    return $retVal
}

add_exporter_config() {
    echo "  calling add_exporter_config:"

    local bridged_network=$(jq --null-input \
                               --arg eth_chain_id "$eth_chain_id" \
      '
        { 
          "Ethereum": { 
            "chainId": $eth_chain_id 
          } 
        }
      '
    )
    # Generate data for Transact (add_exporter_config)
    local bridge_config=$(jq --null-input \
                             --arg bridgehub_para_id "$bridgehub_para_id" \
                             --arg bridged_network "$bridged_network" \
        '
            {
                "bridgeLocation": {
                    "parents": 1,
                    "interior": {
                        "X1": { "Parachain": $bridgehub_para_id }
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

    echo ""
    echo "          bridged_network:"
    echo "${bridged_network}"
    echo ""
    echo "          bridge_config:"
    echo "${bridge_config}"
    echo ""
    echo "--------------------------------------------------"

    local tmp_output_file=$(mktemp)
    generate_hex_encoded_call_data \
      "add-exporter-config" \
      "$statemine_ws_url" \
      "$tmp_output_file" \
      "$bridged_network" \
      "$bridge_config"
    local hex_encoded_data=$(cat $tmp_output_file)
    rm $tmp_output_file

    send_governance_transact_from_relaychain "${statemine_para_id}" "${hex_encoded_data}" 200000000 12000
}

add_universal_alias() {
    echo "  calling add_universal_alias:"
    local location=$(jq --null-input \
                        --arg bridgehub_para_id "$bridgehub_para_id" \
                        '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridgehub_para_id } } } }') # BridgeHub

    local junction=$(jq --null-input \
                        --arg eth_chain_id "$eth_chain_id" \
                        '{ "GlobalConsensus": { "Ethereum": { "chainId": $eth_chain_id } } }')

    echo ""
    echo "          location:"
    echo "${location}"
    echo ""
    echo "          junction:"
    echo "${junction}"
    echo ""
    echo "--------------------------------------------------"

    local tmp_output_file=$(mktemp)
    generate_hex_encoded_call_data "add-universal-alias" "${statemine_ws_url}" "${tmp_output_file}" "$location" "$junction"
    local hex_encoded_data=$(cat $tmp_output_file)
    rm $tmp_output_file

    send_governance_transact_from_relaychain "${statemine_para_id}" "${hex_encoded_data}" 200000000 12000
}

add_reserve_location() {
    echo "  calling add_reserve_location:"

    local nativeTokens=$1
    # Ethereum Native Tokens contract
    local reserve_location=$(jq --null-input \
                        --arg eth_chain_id "$eth_chain_id" \
                        --arg nativeTokens "$nativeTokens" \
                        '{ "V3": {
                            "parents": 2,
                            "interior": {
                                "X2": [
                                    {
                                        "GlobalConsensus": { "Ethereum": { "chainId": $eth_chain_id } },
                                    },
                                    {
                                        "AccountKey20": { "network": { "Ethereum": { "chainId": $eth_chain_id } }, "key": $nativeTokens }
                                    }
                                ]
                            }
                        } }')

    echo ""
    echo "          reserve_location:"
    echo "${reserve_location}"
    echo ""
    echo "--------------------------------------------------"

    local tmp_output_file=$(mktemp)
    generate_hex_encoded_call_data "add-reserve-location" "${statemine_ws_url}" "${tmp_output_file}" "$reserve_location"
    local hex_encoded_data=$(cat $tmp_output_file)
    rm $tmp_output_file

    send_governance_transact_from_relaychain "${statemine_para_id}" "${hex_encoded_data}" 200000000 12000
}

configure_statemine() {
    # Allows messages to be sent from Substrate to ETH.
    add_exporter_config
    # Allow messages to be sent from ETH to Substrate.
    add_universal_alias
    # Allow NativeTokens contract to ReserveAssetDeposited xcm instruction.
    add_reserve_location $(address_for NativeTokens)
}

if [ -z "${from_start_services:-}" ]; then
    echo "configuring statemine"
    configure_statemine
    wait
fi
