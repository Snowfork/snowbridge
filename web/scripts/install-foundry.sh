#!/usr/bin/env bash
# Install Foundry (forge, cast, anvil) for CI/Vercel by downloading the official
# GitHub release tarball. Run from repo root or web/.
# No dependency on the outdated @foundryup/foundry npm package.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Use project-local dir on CI/Vercel so we don't rely on $HOME being writable
if [ -n "${CI:-}" ] || [ -n "${VERCEL:-}" ]; then
  FOUNDRY_DIR="${FOUNDRY_DIR:-$SCRIPT_DIR/.foundry}"
else
  FOUNDRY_DIR="${FOUNDRY_DIR:-$HOME/.foundry}"
fi

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

  TAG="${FOUNDRY_TAG:-v1.5.1}"
  # e.g. foundry_nightly_linux_amd64.tar.gz or foundry_v1.5.1_linux_amd64.tar.gz
  if [ "$TAG" = "nightly" ]; then
    ASSET="foundry_nightly_${OS}_${ARCH}.tar.gz"
  else
    ASSET="foundry_${TAG}_${OS}_${ARCH}.tar.gz"
  fi
  URL="https://github.com/foundry-rs/foundry/releases/download/${TAG}/${ASSET}"

  echo "Installing Foundry from $URL into $FOUNDRY_DIR"
  mkdir -p "$FOUNDRY_DIR"
  curl -sSfL "$URL" -o "/tmp/foundry.tar.gz"
  tar -xzf /tmp/foundry.tar.gz -C "$FOUNDRY_DIR"
  rm -f /tmp/foundry.tar.gz

  # Tarball layout: either one top-level dir (e.g. foundry_nightly_linux_amd64/) with binaries,
  # or binaries directly in FOUNDRY_DIR (darwin). Put forge in FOUNDRY_DIR/bin for a consistent PATH.
  FOUNDRY_BIN="$FOUNDRY_DIR/bin"
  mkdir -p "$FOUNDRY_BIN"
  # Find a subdir that contains forge (skip FOUNDRY_BIN so we don't treat bin as the source dir)
  SUBDIR=$(find "$FOUNDRY_DIR" -maxdepth 1 -type d ! -path "$FOUNDRY_DIR" ! -path "$FOUNDRY_BIN" 2>/dev/null | head -1)

  if [ -n "$SUBDIR" ] && [ -f "$SUBDIR/forge" ]; then
    cp -f "$SUBDIR"/forge "$SUBDIR"/cast "$SUBDIR"/anvil "$FOUNDRY_BIN/" 2>/dev/null || true
    [ -f "$SUBDIR/chisel" ] && cp -f "$SUBDIR/chisel" "$FOUNDRY_BIN/" 2>/dev/null || true
    rm -rf "$SUBDIR"
  fi
  # If binaries were at FOUNDRY_DIR root (e.g. darwin tarball)
  for b in forge cast anvil chisel; do
    if [ -f "$FOUNDRY_DIR/$b" ]; then
      mv -f "$FOUNDRY_DIR/$b" "$FOUNDRY_BIN/$b"
      chmod +x "$FOUNDRY_BIN/$b"
    fi
  done

  if [ ! -f "$FOUNDRY_BIN/forge" ]; then
    echo "Error: forge binary not found after extract. Contents of $FOUNDRY_DIR:"
    ls -la "$FOUNDRY_DIR" 2>/dev/null || true
    exit 1
  fi
  chmod +x "$FOUNDRY_BIN"/*
  export PATH="$FOUNDRY_BIN:$PATH"
fi

# Ensure forge is available
if ! command -v forge >/dev/null 2>&1; then
  echo "Warning: Foundry install failed or forge not in PATH. Skipping forge dependency install."
  exit 1
fi
echo "Foundry installed: $(forge --version)"

# Install forge dependencies. Contracts dir: ../contracts when run from web/, contracts when run from repo root.
CONTRACTS_DIR=""
[ -d "$SCRIPT_DIR/../contracts" ] && CONTRACTS_DIR="$SCRIPT_DIR/../contracts"
[ -d "$SCRIPT_DIR/../../contracts" ] && CONTRACTS_DIR="$SCRIPT_DIR/../../contracts"
[ -d "contracts" ] && CONTRACTS_DIR="contracts"
[ -d "../contracts" ] && CONTRACTS_DIR="../contracts"

if [ -n "$CONTRACTS_DIR" ] && [ -d "$CONTRACTS_DIR" ]; then
  (cd "$CONTRACTS_DIR" && forge build)
fi
