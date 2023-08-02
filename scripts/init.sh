#!/usr/bin/env bash

set -e

echo "Setting up submodules"
git submodule update --init --recursive || true

echo "Setting up git hooks"
ln -sf pre-commit.sh .git/hooks/pre-commit

echo "Installing Rust toolchains"
# https://stackoverflow.com/questions/59895/how-do-i-get-the-directory-where-a-bash-script-is-located-from-within-the-script
script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
"$script_dir"/set-rust-toolchain.sh

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@latest

echo "Installing web packages"
(cd web && pnpm install)
