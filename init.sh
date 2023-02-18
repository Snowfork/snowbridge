
#!/usr/bin/env bash

echo "Initialize foundry libraries from submodule"
git submodule update --init

echo "Install node dependencies"
cd core && pnpm install && cd -

echo "Hack SLOTS_IN_EPOCH in lodestar"
cd core/packages/test && ./scripts/hack-ethereum.sh && cd -

echo "Install husky hook"
./core/node_modules/.bin/husky install
