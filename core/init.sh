#!/usr/bin/env sh

echo "Update submodules"
(cd .. && (git submodule update --init --recursive||true))

echo "Install husky hook"
(cd .. && ./core/node_modules/.bin/husky install)

echo "Initialize foundry libraries"
(cd packages/contracts && (forge install||true))

