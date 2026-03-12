#!/usr/bin/env bash
# Install Foundry (forge, cast, anvil) for CI/Vercel by downloading the official
# GitHub release tarball. Run from repo root or web/.
# No dependency on the outdated @foundryup/foundry npm package.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Use project-local dir on CI/Vercel so we don't rely on $HOME being writable
# if [ -n "${CI:-}" ] || [ -n "${VERCEL:-}" ]; then
#   FOUNDRY_DIR="${FOUNDRY_DIR:-$SCRIPT_DIR/.foundry}"
# else
#   FOUNDRY_DIR="${FOUNDRY_DIR:-$HOME/.foundry}"
# fi
FOUNDRY_DIR="${FOUNDRY_DIR:-$SCRIPT_DIR/.foundry}"

# If forge is already in PATH (e.g. from a previous step), skip install
if command -v forge >/dev/null 2>&1; then
  echo "Foundry (forge) already in PATH: $(forge --version)"
  # Still run forge deps + build below if CONTRACTS_DIR is set
else
  # Detect OS and arch for the release tarball
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  case "$OS" in
    linux)   OS=linux ;;
    darwin)  OS=darwin ;;
    *)       echo "Unsupported OS: $OS"; exit 1 ;;
  esac
  case "$ARCH" in
    x86_64|amd64) ARCH=amd64 ;;
    aarch64|arm64) ARCH=arm64 ;;
    *)       echo "Unsupported arch: $ARCH"; exit 1 ;;
  esac

  TAG="${FOUNDRY_TAG:-nightly}"
  # e.g. foundry_nightly_linux_amd64.tar.gz or foundry_v1.0.0_linux_amd64.tar.gz
  if [ "$TAG" = "nightly" ]; then
    ASSET="foundry_nightly_${OS}_${ARCH}.tar.gz"
  else
    ASSET="foundry_${TAG#v}_${OS}_${ARCH}.tar.gz"
  fi
  URL="https://github.com/foundry-rs/foundry/releases/download/${TAG}/${ASSET}"

  echo "Installing Foundry from $URL into $FOUNDRY_DIR"
  mkdir -p "$FOUNDRY_DIR"
  curl -sSfL "$URL" -o "/tmp/foundry.tar.gz"
  tar -xzf /tmp/foundry.tar.gz -C "$FOUNDRY_DIR"
  rm -f /tmp/foundry.tar.gz

  # Tarball has one top-level dir (e.g. foundry_nightly_linux_amd64) with binaries
  SUBDIR=$(find "$FOUNDRY_DIR" -maxdepth 1 -type d ! -path "$FOUNDRY_DIR" 2>/dev/null | head -1)
  if [ -n "$SUBDIR" ] && [ -f "$SUBDIR/forge" ]; then
    mkdir -p "$FOUNDRY_DIR/bin"
    cp -f "$SUBDIR"/forge "$SUBDIR"/cast "$SUBDIR"/anvil "$FOUNDRY_DIR/bin/" 2>/dev/null || true
    [ -f "$SUBDIR/chisel" ] && cp -f "$SUBDIR/chisel" "$FOUNDRY_DIR/bin/" 2>/dev/null || true
    chmod +x "$FOUNDRY_DIR/bin"/*
    rm -rf "$SUBDIR"
  else
    # Binaries at top level of FOUNDRY_DIR
    mkdir -p "$FOUNDRY_DIR/bin"
    for b in forge cast anvil chisel; do
      [ -f "$FOUNDRY_DIR/$b" ] && mv "$FOUNDRY_DIR/$b" "$FOUNDRY_DIR/bin/" && chmod +x "$FOUNDRY_DIR/bin/$b"
    done
  fi

  export PATH="$FOUNDRY_DIR/bin:$PATH"
fi

# Ensure forge is available
if ! command -v forge >/dev/null 2>&1; then
  echo "Warning: Foundry install failed or forge not in PATH. Skipping forge dependency install."
  exit 0
fi
echo "Foundry installed: $(forge --version)"

# Install forge dependencies (same as scripts/init.sh). Contracts dir: ../contracts when run from web/, contracts when run from repo root.
CONTRACTS_DIR=""
[ -d "$SCRIPT_DIR/../contracts" ] && CONTRACTS_DIR="$SCRIPT_DIR/../contracts"
[ -d "$SCRIPT_DIR/../../contracts" ] && CONTRACTS_DIR="$SCRIPT_DIR/../../contracts"
[ -d "contracts" ] && CONTRACTS_DIR="contracts"
[ -d "../contracts" ] && CONTRACTS_DIR="../contracts"

if [ -n "$CONTRACTS_DIR" ] && [ -d "$CONTRACTS_DIR" ]; then
  echo "Installing forge dependencies in $CONTRACTS_DIR"
  pushd "$CONTRACTS_DIR"
    [ -d "lib" ] && rm -rf lib
    forge install foundry-rs/forge-std --no-git
    forge install https://github.com/dapphub/ds-test --no-git
    forge install https://github.com/Snowfork/canonical-weth --no-git
    forge install https://github.com/PaulRBerg/prb-math --no-git
    forge install https://github.com/OpenZeppelin/openzeppelin-contracts --no-git
  popd
  echo "Forge dependencies installed."
  (cd "$CONTRACTS_DIR" && forge build)
fi
