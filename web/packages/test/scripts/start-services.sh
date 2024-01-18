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

# 6. config bridgehub
echo "Config bridgehub"
source scripts/configure-bridgehub.sh
configure_bridgehub

# 7. config assethub
echo "Config assethub"
source scripts/configure-assethub.sh
configure_assethub

# 8. open hrmp channels
echo "Config hrmp channels"
source scripts/open-hrmp.sh
open_hrmp_channels

#9. config beacon checkpoint for bridgeHub
echo "Config bridgehub beacon,waiting..."
sleep 12 # initialize beacon is a huge transact so sleep to execute in a separate block
source scripts/configure-bridgehub-beacon.sh
configure_bridgehub_beacon

if [ "$skip_relayer" == "false" ]; then
  # 10. start relayer
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
