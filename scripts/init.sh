#!/usr/bin/env bash

set -e

echo "Setting up submodules"
git submodule update --init --recursive || true

echo "Setting up git hooks"
ln -sf pre-commit.sh .git/hooks/pre-commit

echo "Installing Rust toolchains"
# NOTE: This ensures that the toolchain in rust-toolchain.toml is installed.
# rustup has no subcommand (yet) for installing the toolchain in rust-toolchain.toml:
# https://github.com/rust-lang/rustup/issues/2686
# The auto-installation behaviour in rustup will likely be removed:
# https://github.com/rust-lang/rustup/issues/1397
cp rust-toolchain-stable.toml rust-toolchain.toml
rustup show
cp rust-toolchain-nightly.toml rust-toolchain.toml
rustup show
cp rust-toolchain-stable.toml rust-toolchain.toml

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@latest

echo "Installing web packages"
(cd web && pnpm install)
