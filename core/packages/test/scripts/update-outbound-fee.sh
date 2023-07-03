#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
source scripts/xcm-helper.sh

upgrade()
{
    pushd "$contract_dir"
    echo "Deploying upgrade contract..."
    # 0.002ether~=3.75$,unset env variable RELAYER_FEE first
    local relayer_fee="${RELAYER_FEE:-2000000000000000}"
    local response=$(forge create --rpc-url $eth_endpoint_http \
          --constructor-args \
          $(address_for Registry) \
          $relayer_fee \
          --private-key $PRIVATE_KEY \
          --gas-limit $eth_gas_limit \
          src/UpdateOutboundFee.sol:UpdateOutboundFee)
    local upgrade_contract=$(echo $response | awk -F"Deployed to:" '/Deployed to:/{print $2}' | awk -F"Transaction hash" '/Transaction hash/{print $1}' | sed "s/^ *//")
    echo "upgrade contract deployed to:" $upgrade_contract
    # cut params from the last 32*2 characters
    local upgrade_params=$(cast call --rpc-url $eth_endpoint_http $upgrade_contract "createUpgradeParams()"|grep -v "DEBUG"| sed '/^[[:space:]]*$/d' | rev | cut -c 1-64 | rev)
    echo "upgrade params:" 0x"$upgrade_params"
    popd

    echo "Sending upgrade call from relaychain governance..."
    local callindex="3300"
    local upgrade_task_address=$(echo $upgrade_contract | cut -c3-)
    local prefix="0180"
    local upgrade_call="0x$callindex$upgrade_task_address$prefix$upgrade_params"
    echo "upgrade call:" $upgrade_call
    send_governance_transact_from_relaychain $bridgehub_para_id "$upgrade_call" 180000000000 900000
}

if [ -z "${from_start_services:-}" ]; then
    upgrade
fi
