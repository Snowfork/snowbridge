#!/usr/bin/env sh

set -eu


bridgehub_ws_url="${BRIDGEHUB_WS_URL:-bridge-hub-rococo-local}"

trap_message() {
    echo Sending trap message.
    npx polkadot-js-api
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
