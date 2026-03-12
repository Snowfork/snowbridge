#!/usr/bin/env bash
# Install Foundry (forge, cast, anvil) for CI/Vercel. Run from repo root or web/.
# Prefers forge from @foundryup/foundry (pnpm install) when available; otherwise uses foundryup.
set -e

# Prefer forge from node_modules (e.g. @foundryup/foundry) when script runs after pnpm install
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR=""
[ -d "$SCRIPT_DIR/node_modules/.bin" ] && WEB_DIR="$SCRIPT_DIR"
[ -d "$SCRIPT_DIR/../node_modules/.bin" ] && WEB_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
[ -n "$WEB_DIR" ] && export PATH="$WEB_DIR/node_modules/.bin:$PATH"

# Use project-local dir on CI/Vercel so we don't rely on $HOME being writable
if [ -n "${CI:-}" ] || [ -n "${VERCEL:-}" ]; then
  FOUNDRY_DIR="${FOUNDRY_DIR:-$SCRIPT_DIR/.foundry}"
else
  FOUNDRY_DIR="${FOUNDRY_DIR:-$HOME/.foundry}"
fi
mkdir -p "$FOUNDRY_DIR/bin"

# Skip foundryup if forge is already available (e.g. from npm package)
if command -v forge >/dev/null 2>&1; then
  echo "Foundry (forge) already in PATH: $(forge --version)"
else
  # Download foundryup into FOUNDRY_DIR/bin
  curl -sSfL "https://raw.githubusercontent.com/foundry-rs/foundry/HEAD/foundryup/foundryup" -o "$FOUNDRY_DIR/bin/foundryup"
  chmod +x "$FOUNDRY_DIR/bin/foundryup"
  export PATH="$FOUNDRY_DIR/bin:$PATH"
  CI=1 FOUNDRY_DIR="$FOUNDRY_DIR" foundryup --no-modify-path 2>/dev/null || true
fi

# Ensure forge is available. If not (e.g. restricted env), exit successfully so build can continue.
if ! command -v forge >/dev/null 2>&1; then
  echo "Warning: Foundry install failed or forge not in PATH. Skipping forge dependency install."
  exit 0
fi
echo "Foundry installed: $(forge --version)"

# Install forge dependencies (same as scripts/init.sh). Contracts dir: ../contracts when run from web/, contracts when run from repo root.
CONTRACTS_DIR=""
[ -d "../contracts" ] && CONTRACTS_DIR="../contracts"
[ -d "contracts" ] && CONTRACTS_DIR="contracts"
if [ -n "$CONTRACTS_DIR" ]; then
  echo "Installing forge dependencies in $CONTRACTS_DIR"
  pushd "$CONTRACTS_DIR"
    # Remove existing lib so forge install does not hit "destination path already exists"
    [ -d "lib" ] && rm -rf lib
    forge install foundry-rs/forge-std --no-git
    forge install https://github.com/dapphub/ds-test --no-git
    forge install https://github.com/Snowfork/canonical-weth --no-git
    forge install https://github.com/PaulRBerg/prb-math --no-git
    forge install https://github.com/OpenZeppelin/openzeppelin-contracts --no-git
  popd
  echo "Forge dependencies installed."
fi
