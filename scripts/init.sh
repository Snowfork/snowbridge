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
# This auto-installation behaviour in rustup will likely change:
# https://github.com/rust-lang/rustup/issues/1397
rustup show
rustup install --profile minimal nightly-2023-07-31
rustup component add rustfmt --toolchain nightly-2023-07-31

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@latest

echo "Installing web packages"
(cd web && pnpm install)
