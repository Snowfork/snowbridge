#!/usr/bin/env bash
set -eu

start=$(date +%s)

from_start_services=true

source scripts/set-env.sh
source scripts/build-binary.sh

trap kill_all SIGINT SIGTERM EXIT
cleanup

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
source scripts/configure-beefy.sh
configure_beefy

# 4. config beacon client
source scripts/configure-beacon.sh
configure_beacon

# 5. Configure bridgehub exporter on statemine
source scripts/configure-statemine.sh
configure_statemine

# 6. Configure relayers
source scripts/start-relayer.sh
echo "Config relayers"
config_relayer

if [ "$skip_relayer" == "false" ]; then
    # 7. start relayer
    echo "Starting relayers"
    start_relayer
fi

echo "Testnet has been initialized"

end=$(date +%s)
runtime=$((end-start))
minutes=$(( (runtime % 3600) / 60 ));
seconds=$(( (runtime % 3600) % 60 ));
echo "Took $minutes minutes $seconds seconds"

wait
