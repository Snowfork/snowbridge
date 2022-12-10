#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/build-binary.sh
source scripts/start-chains.sh
source scripts/start-relayer.sh

trap forcekill SIGINT SIGTERM EXIT

# 0. check required tools
check_tool

# 1. forcekill and install binary if not exist
echo "Installing binaries if not exist"
forcekill && install_binary

# 2. start ethereum and polkadot chains
echo "Starting ethereum and polkadot chains"
start_chains

# 3. start relayer
start_relayer

# 4. waiting sync headers 
until grep "starting to sync finalized headers" beacon-relay.log > /dev/null; do
    echo "Waiting for beacon relay to sync headers..."
    sleep 5
done

echo "Testnet has been initialized"

wait
