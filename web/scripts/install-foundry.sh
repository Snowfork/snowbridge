#!/usr/bin/env bash
# Install Foundry (forge, cast, anvil) for CI/Vercel. Run from repo root or web/.
set -e

FOUNDRY_DIR="${FOUNDRY_DIR:-$HOME/.foundry}"
mkdir -p "$FOUNDRY_DIR/bin"

# Download foundryup into FOUNDRY_DIR/bin
curl -sSfL "https://raw.githubusercontent.com/foundry-rs/foundry/HEAD/foundryup/foundryup" -o "$FOUNDRY_DIR/bin/foundryup"
chmod +x "$FOUNDRY_DIR/bin/foundryup"

export PATH="$FOUNDRY_DIR/bin:$PATH"
# Non-interactive install (skip profile modification, install latest release)
CI=1 FOUNDRY_DIR="$FOUNDRY_DIR" foundryup --no-modify-path 2>/dev/null || true

# Ensure forge is available
if ! command -v forge >/dev/null 2>&1; then
  echo "Foundry install failed or forge not in PATH. Add to build: export PATH=\"$FOUNDRY_DIR/bin:\$PATH\""
  exit 1
fi
echo "Foundry installed: $(forge --version)"

# Install forge dependencies (same as scripts/init.sh). Contracts dir: ../contracts when run from web/, contracts when run from repo root.
CONTRACTS_DIR=""
[ -d "../contracts" ] && CONTRACTS_DIR="../contracts"
[ -d "contracts" ] && CONTRACTS_DIR="contracts"
if [ -n "$CONTRACTS_DIR" ]; then
  echo "Installing forge dependencies in $CONTRACTS_DIR"
  pushd "$CONTRACTS_DIR"
    forge install foundry-rs/forge-std --no-git
    forge install https://github.com/dapphub/ds-test --no-git
    forge install https://github.com/Snowfork/canonical-weth --no-git
    forge install https://github.com/PaulRBerg/prb-math --no-git
    forge install https://github.com/OpenZeppelin/openzeppelin-contracts --no-git
  popd
  echo "Forge dependencies installed."
fi
