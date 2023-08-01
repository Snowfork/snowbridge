#!/usr/bin/env bash

set -e

echo "Setting up submodules"
git submodule update --init --recursive || true

echo "Setting up git hooks"
ln -sf pre-commit.sh .git/hooks/pre-commit

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@latest

echo "Installing web packages"
(cd web && pnpm install)
