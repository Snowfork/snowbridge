#!/usr/bin/env bash
# Build contract-types: run forge + typechain + tsc when forge and contracts/ exist;
# otherwise run only tsc (requires generated src/ to exist, e.g. committed or from cache).
set -e
cd "$(dirname "$0")/.."
CONTRACTS_DIR="../../../contracts"
if command -v forge >/dev/null 2>&1 && [ -d "$CONTRACTS_DIR" ]; then
  rm -rf src dist
  cd "$CONTRACTS_DIR" && forge build && cd "../web/packages/contract-types"
  pnpm typechain
  tsc --build --force
elif [ -d "src" ]; then
  tsc --build --force
else
  echo "contract-types: need either (forge in PATH + $CONTRACTS_DIR) or existing src/"
  exit 1
fi
