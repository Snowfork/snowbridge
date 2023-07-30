#!/usr/bin/env bash

set -e

git submodule update --init --recursive || true
(cd web && pnpm install)
go install github.com/ferranbt/fastssz/sszgen@latest
ln -sf pre-commit.sh .git/hooks/pre-commit
