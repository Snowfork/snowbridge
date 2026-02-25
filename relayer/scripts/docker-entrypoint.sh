#!/bin/bash
set -e

# Process config file with environment variable substitution
# Looks for --config argument and substitutes env vars in that file

CONFIG_FILE=""
ARGS=()

for arg in "$@"; do
    if [[ "$arg" == --config=* ]]; then
        CONFIG_FILE="${arg#--config=}"
    elif [[ "$prev_arg" == "--config" ]]; then
        CONFIG_FILE="$arg"
    fi
    ARGS+=("$arg")
    prev_arg="$arg"
done

if [[ -n "$CONFIG_FILE" && -f "$CONFIG_FILE" ]]; then
    # Create processed config directory
    mkdir -p /tmp/config

    # Get the filename
    CONFIG_BASENAME=$(basename "$CONFIG_FILE")
    PROCESSED_CONFIG="/tmp/config/$CONFIG_BASENAME"

    # Substitute environment variables
    envsubst < "$CONFIG_FILE" > "$PROCESSED_CONFIG"

    # Replace config path in arguments
    NEW_ARGS=()
    skip_next=false
    for arg in "${ARGS[@]}"; do
        if $skip_next; then
            NEW_ARGS+=("$PROCESSED_CONFIG")
            skip_next=false
        elif [[ "$arg" == --config=* ]]; then
            NEW_ARGS+=("--config=$PROCESSED_CONFIG")
        elif [[ "$arg" == "--config" ]]; then
            NEW_ARGS+=("$arg")
            skip_next=true
        else
            NEW_ARGS+=("$arg")
        fi
    done

    exec /usr/local/bin/snowbridge-relay "${NEW_ARGS[@]}"
else
    exec /usr/local/bin/snowbridge-relay "$@"
fi
