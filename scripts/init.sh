#!/usr/bin/env bash

set -e

echo "Setting up submodules"
git submodule update --init --recursive || true

echo "Setting up git hooks"
ln -sf pre-commit.sh .git/hooks/pre-commit

echo "Installing Rust toolchains"
# NOTE: Switch back to the toolchain file once it supports explicit installation.
# rustup has no subcommand (yet) for installing the toolchain in rust-toolchain.toml:
# https://github.com/rust-lang/rustup/issues/2686
# The auto-installation behaviour in rustup will likely change:
# https://github.com/rust-lang/rustup/issues/1397
rustup install --profile minimal 1.70.0
rustup default 1.70.0
rustup target add --toolchain 1.70.0 wasm32-unknown-unknown
rustup component add --toolchain 1.70.0 clippy rust-analyzer rust-src
rustup install --profile minimal nightly-2023-07-31
rustup component add --toolchain nightly-2023-07-31 rustfmt

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@latest

echo "Installing web packages"
(cd web && pnpm install)
