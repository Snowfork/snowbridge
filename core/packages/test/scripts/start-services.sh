#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/start-chains.sh
source scripts/configure-contracts.sh
source scripts/start-relayer.sh

trap forcekill SIGINT SIGTERM EXIT
# 0. check required tools | check binaries | cleanup resource
check_build_tool && check_binary && cleanup
# 1. start ethereum and polkadot chains
echo "Starting ethereum and polkadot chains"
start_chains
echo "Waiting for consensus between polkadot and parachain"
sleep 30
# 2. initialize bridge contracts
configure_contracts
# 3. start relayer
start_relayer

# 4. waiting sync headers 
until grep "starting to sync finalized headers" beacon-relay.log > /dev/null; do
    echo "Waiting for beacon relay to sync headers..."
    sleep 5
done

echo "Testnet has been initialized"

wait
