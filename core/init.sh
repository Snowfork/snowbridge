#!/usr/bin/env bash

echo "Install husky hook"
(cd .. && ./core/node_modules/.bin/husky install)

echo "Initialize foundry libraries"
(cd packages/contracts && (forge install||true))
