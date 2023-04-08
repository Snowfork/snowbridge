#!/usr/bin/env sh

set -eu

seed="${BRIDGEHUB_WS_URL:-//Ferdie}"

bridge_hub_ws_url="${BRIDGE_HUB_WS_URL:-ws://127.0.0.1:11144}"
bridge_hub_para_id="${BRIDGE_HUB_PARA_ID:-1013}"

statemine_ws_url="${STATEMINE_WS_URL:-ws://127.0.0.1:12144}"
statemine_para_id="${BRIDGE_HUB_PARA_ID:-1000}"

trap_message() {
    echo Sending trap message.

    echo "  calling send_xcm_trap_from_statemine:"
    echo "      seed: ${seed}"
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
                              "ExportMessage": {
                                "network": { 
                                    "Ethereum": { chainId: 1337 }
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
        --seed "${seed?}" \
        tx.polkadotXcm.send \
            "${dest}" \
            "${message}"
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
    trap)
      trap_message
      shift
      ;;
    *)
      echo "Unknown message: $1"
      exit 1
      ;;
  esac
done

exit 0
