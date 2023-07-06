#!/usr/bin/env bash
set -eu

start=$(date +%s)

from_start_services=true

source scripts/xcm-helper.sh
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
else
  source scripts/start-goerli.sh
  # deploy beacon node locally for fast response time or retrieving beacon state from remote could be very slow
  deploy_goerli
fi

# 3. deploy contracts
echo "Deploying ethereum contracts"
source scripts/deploy-contracts.sh
deploy_contracts &

# 4. start polkadot
echo "Starting polkadot nodes"
source scripts/deploy-polkadot.sh
deploy_polkadot

# wait for contract deployed
echo "Waiting contract deployed"
wait_contract_deployed

# 5. fund substrate accounts
echo "Funding substrate accounts"
transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $statemine_sovereign_account
transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $beacon_relayer_pub_key
transfer_balance $relaychain_ws_url "//Charlie" 1013 1000000000000000 $execution_relayer_pub_key
transfer_balance $relaychain_ws_url "//Charlie" 1000 1000000000000000 $registry_contract_sovereign_account

# 6. config beefy client
echo "Config beefy client"
source scripts/configure-beefy.sh
configure_beefy

# 7. config bridgehub
echo "Config bridgehub"
source scripts/configure-bridgehub.sh
configure_bridgehub

if [ "$skip_relayer" == "false" ]; then
    # 8. start relayer
    echo "Starting relayers"
    source scripts/start-relayer.sh
    deploy_relayer
fi

echo "Testnet has been initialized"

end=$(date +%s)
runtime=$((end-start))
minutes=$(( (runtime % 3600) / 60 ));
seconds=$(( (runtime % 3600) % 60 ));
echo "Took $minutes minutes $seconds seconds"

wait
