#!/usr/bin/env bash
set -eu

start=$(date +%s)

source scripts/set-env.sh
source scripts/build-binary.sh
from_start_services=true

trap kill_all SIGINT SIGTERM EXIT
kill_all && cleanup
# 0. check required tools
echo "Check building tools"
check_tool

# 1. install binary if required
echo "Installing binaries if required"
install_binary

# 2. start ethereum and polkadot chains
echo "Starting ethereum and polkadot chains"
source scripts/start-chains.sh
start_chains

# 3. config beefy client
source scripts/configure-contracts.sh
configure_contracts

# 4. start relayer
echo "Starting relayers"
source scripts/start-relayer.sh
start_relayer

echo "Testnet has been initialized"

end=$(date +%s)
runtime=$((end-start))
minutes=$(( (runtime % 3600) / 60 ));
seconds=$(( (runtime % 3600) % 60 ));
echo "Took $minutes minutes $seconds seconds"

wait
