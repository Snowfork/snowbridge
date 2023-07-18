#!/usr/bin/env bash

echo "Update submodules"
(cd .. && (git submodule update --init --recursive||true))

if [ ! -d ../.husky/_/ ]; then
    echo "Install husky hook"
    (cd .. && ./core/node_modules/.bin/husky install)
else
    echo "Found husky hook"
fi

if [ ! -f "$(command -v sszgen)" ]; then
    echo "Installing sszgen"
    go install github.com/ferranbt/fastssz/sszgen@latest
else
    echo "Found sszgen"
fi

echo "Initialize foundry libraries"
(cd packages/contracts && forge install)
