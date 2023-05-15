#!/usr/bin/env sh

set -eu
source scripts/set-env.sh

statemine_trap_message() {
    echo Sending trap message.

    echo "  calling send_xcm_trap_from_statemine:"
    echo "      seed: ${sudo_seed}"
    echo "      statemine_ws_url: ${statemine_ws_url}"
    echo "      bridge_hub_ws_url: ${bridge_hub_ws_url}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      statemin_para_id: ${statemine_para_id}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg bridge_hub_para_id "$bridge_hub_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridge_hub_para_id } } } }')

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
        --seed "${sudo_seed?}" \
        tx.polkadotXcm.send \
            "${dest}" \
            "${message}"
}

bridgehub_trap_message() {
    echo Sending trap message.

    echo "  calling send_xcm_trap_from_statemine:"
    echo "      seed: ${sudo_seed}"
    echo "      statemine_ws_url: ${statemine_ws_url}"
    echo "      bridge_hub_ws_url: ${bridge_hub_ws_url}"
    echo "      bridge_hub_para_id: ${bridge_hub_para_id}"
    echo "      statemin_para_id: ${statemine_para_id}"
    echo "      params:"

    local dest=$(jq --null-input \
                    --arg bridge_hub_para_id "$bridge_hub_para_id" \
                    '{ "V3": { "parents": 1, "interior": { "X1": { "Parachain": $bridge_hub_para_id } } } }')

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
        --ws "${bridge_hub_ws_url?}" \
        --seed "${sudo_seed?}" \
        tx.polkadotXcm.execute \
            "${message}" \
            "${weight}"
}

if [ "$#" -eq 0 ]; then
    cat <<EOF
usage:
    $0 messages

    messages:
        trap - Sends an xcm message from Statemine to Bridgehub and then does an XCM trap.
EOF
    exit 1
fi

while [ "$#" -gt 0 ]; do
  case "$1" in
    statemine-trap)
      shift
      statemine_trap_message
      ;;
    bridgehub-trap)
      shift
      bridgehub_trap_message
      ;;
    *)
      echo "Unknown message: $1"
      exit 1
      ;;
  esac
done

exit 0
