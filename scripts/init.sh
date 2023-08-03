#!/usr/bin/env bash

set -e

echo "Setup git hooks"
git config --local core.hooksPath hooks/

echo "Update submodules"
git submodule update --init --recursive || true

echo "Installing dev tools"
go install github.com/ferranbt/fastssz/sszgen@v0.1.3

echo "Install node packages"
(cd web && pnpm install)
