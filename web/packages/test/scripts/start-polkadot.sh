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
build_binaries
build_relayer

# 2. start polkadot
echo "Starting polkadot nodes"
source scripts/deploy-polkadot.sh
deploy_polkadot

# 4. generate beefy checkpoint
echo "Generate beefy checkpoint"
source scripts/generate-beefy-checkpoint.sh
generate_beefy_checkpoint

# 6. config substrate
echo "Config Substrate"
source scripts/configure-substrate.sh
configure_substrate

echo "Prod testnet has been initialized"

end=$(date +%s)
runtime=$((end - start))
minutes=$(((runtime % 3600) / 60))
seconds=$(((runtime % 3600) % 60))
echo "Took $minutes minutes $seconds seconds"

wait
