#!/usr/bin/env bash

set -euo pipefail

# Wrapper for running Anvil as a local mainnet fork.
#
# Usage:
#   MAINNET_RPC_URL=https://... ./start-anvil-mainnet-fork.sh
#   ./start-anvil-mainnet-fork.sh --rpc-url https://... --block-number 22000000
#
# Optional env vars:
#   MAINNET_RPC_URL (default: https://ethereum.publicnode.com)
#   ANVIL_HOST (default: 0.0.0.0)
#   ANVIL_PORT (default: 8545)
#   ANVIL_CHAIN_ID (default: 1)
#   ANVIL_ACCOUNTS (default: 20)
#   ANVIL_BALANCE_ETHER (default: 10000)
#   REFRESH_INTERVAL (default: 600 seconds; set to 0 to disable)

DEFAULT_MAINNET_RPC_URL="https://ethereum.publicnode.com"
RPC_URL="${MAINNET_RPC_URL:-${ETH_MAINNET_RPC_URL:-$DEFAULT_MAINNET_RPC_URL}}"
BLOCK_NUMBER=""
ANVIL_HOST="${ANVIL_HOST:-0.0.0.0}"
ANVIL_PORT="${ANVIL_PORT:-8545}"
ANVIL_CHAIN_ID="${ANVIL_CHAIN_ID:-1}"
ANVIL_ACCOUNTS="${ANVIL_ACCOUNTS:-20}"
ANVIL_BALANCE_ETHER="${ANVIL_BALANCE_ETHER:-10000}"
REFRESH_INTERVAL="${REFRESH_INTERVAL:-600}"

usage() {
    cat <<'EOF'
Usage: start-anvil-mainnet-fork.sh [options]

Options:
    --rpc-url URL                Mainnet RPC endpoint (or use MAINNET_RPC_URL / ETH_MAINNET_RPC_URL)
  --block-number NUMBER          Pin the fork to an exact block (disables periodic refresh)
  --refresh-interval SECONDS     Re-fork from upstream every N seconds (default: 600; 0 = disabled)
  -h, --help                     Show this help

Default RPC URL:
    https://ethereum.publicnode.com
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rpc-url)
            RPC_URL="${2:-}"
            shift 2
            ;;
        --block-number)
            BLOCK_NUMBER="${2:-}"
            shift 2
            ;;
        --refresh-interval)
            REFRESH_INTERVAL="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage
            exit 1
            ;;
    esac
done

if ! command -v anvil >/dev/null 2>&1; then
    echo "Error: anvil not found in PATH. Install Foundry first: https://book.getfoundry.sh/getting-started/installation" >&2
    exit 1
fi

# A fixed --block-number means the user wants a frozen state — never refresh.
if [[ -n "$BLOCK_NUMBER" ]]; then
    REFRESH_INTERVAL=0
fi

CMD=(
    anvil
    --host "$ANVIL_HOST"
    --port "$ANVIL_PORT"
    --chain-id "$ANVIL_CHAIN_ID"
    --accounts "$ANVIL_ACCOUNTS"
    --balance "$ANVIL_BALANCE_ETHER"
    --fork-url "$RPC_URL"
)

if [[ -n "$BLOCK_NUMBER" ]]; then
    CMD+=(--fork-block-number "$BLOCK_NUMBER")
fi

echo "Starting Anvil mainnet fork on ${ANVIL_HOST}:${ANVIL_PORT} (chain id ${ANVIL_CHAIN_ID})"
if [[ -n "$BLOCK_NUMBER" ]]; then
    echo "Fork block: ${BLOCK_NUMBER} (frozen — periodic refresh disabled)"
elif [[ "$REFRESH_INTERVAL" -gt 0 ]]; then
    echo "Periodic refresh: every ${REFRESH_INTERVAL}s (anvil_reset to upstream HEAD)"
else
    echo "Periodic refresh: disabled"
fi

# Background a refresh loop that re-forks against the latest upstream block.
# It exits cleanly once Anvil's port stops responding (i.e. Anvil shut down).
if [[ "$REFRESH_INTERVAL" -gt 0 ]]; then
    REFRESH_URL="http://127.0.0.1:${ANVIL_PORT}"
    (
        # Give Anvil time to bind its port before the first reset.
        sleep "$REFRESH_INTERVAL"
        while curl -sf -m 5 -X POST -H "Content-Type: application/json" \
                --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
                "$REFRESH_URL" >/dev/null 2>&1; do
            payload=$(printf '{"jsonrpc":"2.0","method":"anvil_reset","params":[{"forking":{"jsonRpcUrl":"%s"}}],"id":1}' "$RPC_URL")
            if curl -sf -m 30 -X POST -H "Content-Type: application/json" \
                    --data "$payload" "$REFRESH_URL" >/dev/null 2>&1; then
                echo "[refresh] anvil_reset to upstream HEAD at $(date '+%H:%M:%S')"
            else
                echo "[refresh] anvil_reset failed at $(date '+%H:%M:%S')" >&2
            fi
            sleep "$REFRESH_INTERVAL"
        done
    ) &
    REFRESH_PID=$!
    # Stop the refresher when this script exits (Ctrl-C or Anvil dies).
    trap 'kill "$REFRESH_PID" 2>/dev/null || true' EXIT
fi

exec "${CMD[@]}"
