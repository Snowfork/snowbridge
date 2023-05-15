#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

config_beacon_checkpoint()
{
    check_point_call=$($relay_bin generate-beacon-checkpoint --spec $active_spec --url $beacon_endpoint_http)
    send_governance_transact "ws://127.0.0.1:9944" "//Alice" 1013 "$check_point_call" 180000000000 900000
}

config_beacon_relayer()
{
    # Configure beacon relay
    jq \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --arg active_spec $active_spec \
    '
      .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.activeSpec = $active_spec
    ' \
    config/beacon-relay.json > $output_dir/beacon-relay.json
}

wait_beacon_chain_ready()
{
    initial_beacon_block=""
    while [ -z "$initial_beacon_block" ] || [ "$initial_beacon_block" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]
    do
        echo "Waiting for beacon chain to finalize to get initial block..."
        initial_beacon_block=$(curl -s "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" \
            | jq -r '.data.finalized.root' || true)
        sleep 3
    done
}

# todo: refactoring as common
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
        --ws "${relay_url?}" \
        --seed "${relay_chain_seed?}" \
        --sudo \
        tx.xcmPallet.send \
            "${dest}" \
            "${message}"
}

function configure_beacon()
{
    wait_beacon_chain_ready
    config_beacon_relayer
    config_beacon_checkpoint
}

if [ -z "${from_start_services:-}" ]; then
    echo "config beacon checkpoint only!"
    wait_beacon_chain_ready
    config_beacon_relayer
    config_beacon_checkpoint
    wait
fi
