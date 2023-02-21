
#!/usr/bin/env bash

echo "Hack SLOTS_IN_EPOCH in lodestar"
cd packages/test && ./scripts/hack-ethereum.sh && cd -

echo "Install husky hook"
cd .. && ./core/node_modules/.bin/husky install && cd -

echo "Initialize foundry libraries"
cd packages/contracts && (forge install||true) && cd -
