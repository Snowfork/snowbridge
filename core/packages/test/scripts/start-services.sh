#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/start-chains.sh
source scripts/configure-contracts.sh
source scripts/start-relayer.sh

trap trapkill SIGINT SIGTERM EXIT
# 0. check binaries and cleanup resource
check_binary && cleanup
# 1. start ethereum and polkadot chains
echo "Starting ethereum and polkadot chains"
start_chains
echo "Waiting for consensus between polkadot and parachain"
sleep 60
# 2. initialize bridge contracts
configure_contracts
# 3. start relayer
start_relayer

echo "Process Tree:"
pstree -T $$

# 4. waiting sync headers 
until grep "starting to sync finalized headers" beacon-relay.log > /dev/null; do
    echo "Waiting for beacon relay to sync headers..."
    sleep 5
done

echo "Testnet has been initialized"

wait
