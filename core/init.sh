#!/usr/bin/env bash

echo "Hack SLOTS_IN_EPOCH in lodestar"
(cd packages/test && ./scripts/hack-ethereum.sh)

echo "Install husky hook"
(cd .. && ./core/node_modules/.bin/husky install)

echo "Initialize foundry libraries"
(cd packages/contracts && (forge install||true))
