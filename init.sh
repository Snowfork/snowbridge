
#!/usr/bin/env bash

cd core && pnpm install && cd -

echo "Hack SLOTS_IN_EPOCH in lodestar"
cd core/packages/test && ./scripts/hack-ethereum.sh && cd -

echo "Install foundry libraries"
cd core/packages/contracts && forge install && cd -

echo "Install husky hook"
./core/node_modules/.bin/husky install
