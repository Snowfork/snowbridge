#!/usr/bin/env sh

set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

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
