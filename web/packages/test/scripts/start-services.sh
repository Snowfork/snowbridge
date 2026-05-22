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

# 2. start ethereum
echo "Starting ethereum nodes"
if [ "$eth_network" == "localhost" ]; then
  source scripts/deploy-ethereum.sh
  deploy_ethereum
fi

# 3. start polkadot
echo "Starting polkadot nodes"
source scripts/deploy-polkadot.sh
deploy_polkadot

# 4. generate beefy checkpoint
echo "Generate beefy checkpoint"
source scripts/generate-beefy-checkpoint.sh
generate_beefy_checkpoint

# 5. deploy contracts
echo "Deploying ethereum contracts"
source scripts/deploy-contracts.sh
deploy_contracts

# 6. start beacon state service
# The beacon state service is actually required for beacon checkpoint initialization, so it should be moved before that step.
echo "Starting beacon state service"
source scripts/deploy-beacon-state.sh
deploy_beacon_state_service
# Wait for beacon state service to be ready
sleep 5

# 7. config substrate
# For beacon checkpoint initialization and other required setup for all tests.
echo "Config Substrate"
source scripts/configure-substrate.sh
configure_substrate

# 8. config others,
# This mainly involves configuring Penpal for third-party parachain integrations
# and running some advanced tests, which takes considerable time to set up.
# Since most tests do not require this, we should make it optional to configure.
# It can be run manually when needed.
if [ "$skip_penpal_config" == "false" ]; then
  echo "Config Others"
  source scripts/configure-others.sh
  configure_all
fi

# 9. start relayer
# this is for daily relayer testing, can run it manually if needed
if [ "$skip_relayer" == "false" ]; then
  echo "Starting relayers"
  source scripts/start-relayer.sh
  deploy_relayer
fi

echo "Testnet has been initialized"

end=$(date +%s)
runtime=$((end - start))
minutes=$(((runtime % 3600) / 60))
seconds=$(((runtime % 3600) % 60))
echo "Took $minutes minutes $seconds seconds"

wait
