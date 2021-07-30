#!/usr/bin/env bash
set -eu

# config directory
configdir=/tmp/snowbridge-e2e-config
rm -rf $configdir
mkdir $configdir

# kill all potentially old processes
kill $(ps -aux | grep -e polkadot/target -e ganache-cli -e snowbridge-relay -e release/snowbridge | awk '{print $2}') || true


address_for()
{
    cat $configdir/contracts.json | jq -r .contracts.${1}.address
}

start_ganache()
{
    echo "Starting Ganache"

    npx ganache-cli \
        --port=8545 \
        --blockTime 12 \
        --networkId=344 \
        --chainId=344 \
        --deterministic \
        --db $configdir/ganache.db \
        --mnemonic="stone speak what ritual switch pigeon weird dutch burst shaft nature shove" \
        --gasLimit=8000000 \
        >ganache.log 2>&1 &

    scripts/wait-for-it.sh -t 32 localhost:8545
    sleep 5
}

deploy_contracts()
{
    echo "Deploying contracts"
    pushd ../ethereum

    npx hardhat deploy --network localhost --reset --export $configdir/contracts.json

    popd

    echo "Wrote configuration to $configdir"
}

start_polkadot_launch()
{
    echo "Building parachain and starting polkadot launch"
    pushd ../parachain
    bin=$(pwd)/target/release/snowbridge

    cargo build --release --no-default-features --features with-local-runtime

    echo "Generating Parachain spec"
    target/release/snowbridge build-spec --disable-default-bootnode > $configdir/spec.json

    echo "Inserting Ganache chain info into genesis spec"
    ethereum_initial_header=$(curl http://localhost:8545 \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
        | node ../test/scripts/helpers/transformEthHeader.js)
    node ../test/scripts/helpers/overrideParachainSpec.js $configdir/spec.json \
        genesis.runtime.ethereumLightClient.initialDifficulty 0x0 \
        genesis.runtime.ethereumLightClient.initialHeader "$ethereum_initial_header" \
        genesis.runtime.parachainInfo.parachainId 1000 \
        para_id 1000

    if [ $# -eq 1 ] && [ $1 = "malicious" ]; then
        jq '.genesis.runtime.dotApp.address = "0x433488cec14C4478e5ff18DDC7E7384Fc416f148"' \
          $configdir/spec.json > spec.malicious.json && \
          mv spec.malicious.json $configdir/spec.json
    fi

    echo "Writing Polkadot configuration"
    polkadotbinary=/tmp/polkadot/target/release/polkadot
    if [[ -f ../test/.env ]]; then
        source ../test/.env
    fi

    if [ $# -eq 1 ] && [ $1 = "duplicate" ];
    then
        echo "Generating second parachain spec"
        target/release/snowbridge build-spec --disable-default-bootnode > $configdir/spec2.json

        node ../test/scripts/helpers/overrideParachainSpec.js $configdir/spec2.json \
            genesis.runtime.ethereumLightClient.initialDifficulty 0x0 \
            genesis.runtime.ethereumLightClient.initialHeader "$ethereum_initial_header" \
            genesis.runtime.parachainInfo.parachainId 1001 \
            para_id 1001
        jq  -s '.[0] * .[1]' config-dup.json ../test/config/launchConfigOverridesDup.json \
            | jq ".parachains[0].bin = \"$bin\"" \
            | jq ".parachains[0].chain = \"$configdir/spec.json\"" \
            | jq ".parachains[1].bin = \"$bin\"" \
            | jq ".parachains[1].chain = \"$configdir/spec2.json\"" \
            | jq ".relaychain.bin = \"$polkadotbinary\"" \
            > $configdir/polkadotLaunchConfig.json
    else
        jq  -s '.[0] * .[1]' config.json ../test/config/launchConfigOverrides.json \
            | jq ".parachains[0].bin = \"$bin\"" \
            | jq ".parachains[0].chain = \"$configdir/spec.json\"" \
            | jq ".relaychain.bin = \"$polkadotbinary\"" \
            > $configdir/polkadotLaunchConfig.json
    fi

    polkadot-launch $configdir/polkadotLaunchConfig.json &

    popd

    scripts/wait-for-it.sh -t 120 localhost:11144
}

start_relayer()
{
    echo "Generate Relay config"

    jq \
        --arg k1 $(address_for BeefyLightClient) \
    '
      .ethereum.contracts.BeefyLightClient = $k1
    ' \
    config/beefy-relay.json > $configdir/beefy-relay.json

    jq \
        --arg k1 $(address_for BasicInboundChannel) \
        --arg k2 $(address_for IncentivizedInboundChannel) \
        --arg k3 $(address_for BeefyLightClient) \
    '
      .ethereum.contracts.BasicInboundChannel = $k1
    | .ethereum.contracts.IncentivizedInboundChannel = $k2
    | .ethereum.contracts.BeefyLightClient = $k3
    ' \
    config/parachain-relay.json > $configdir/parachain-relay.json

    jq \
        --arg k1 $(address_for BasicOutboundChannel) \
        --arg k2 $(address_for IncentivizedOutboundChannel) \
    '
      .ethereum.contracts.BasicOutboundChannel = $k1
    | .ethereum.contracts.IncentivizedOutboundChannel = $k2
    ' \
    config/ethereum-relay.json > $configdir/ethereum-relay.json

    # Build relay
    mage -d ../relayer build

    # Launch beefy relay
    (
        > beefy-relay.log
        while true
        do
            ../relayer/build/snowbridge-relay run beefy \
                --config $configdir/beefy-relay.json \
                --private-key "0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109" \
                >>beefy-relay.log 2>&1 || true
            echo "Beefy relay died. Restarting after delay..."
            sleep 20
        done
    ) &

    # Launch parachain relay
    (
        > parachain-relay.log
        while true
        do
            ../relayer/build/snowbridge-relay run parachain \
                --config $configdir/parachain-relay.json \
                --private-key "0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a" \
                >>parachain-relay.log 2>&1 || true
            echo "Parachain relay died. Restarting after delay..."
            sleep 20
        done
    ) &

    # Launch ethereum relay
    (
        > ethereum-relay.log
        while true
        do
            ../relayer/build/snowbridge-relay run ethereum \
                --config $configdir/ethereum-relay.json \
                --private-key "//Relay" \
                >>ethereum-relay.log 2>&1 || true
            echo "Ethereum relay died. Restarting after delay..."
            sleep 20
        done
    ) &

}

cleanup() {
    kill $(jobs -p)
    kill $(ps -aux | grep -e polkadot/target -e ganache-cli -e snowbridge-relay -e release/snowbridge | awk '{print $2}') || true
}

trap cleanup SIGINT SIGTERM EXIT

start_ganache
deploy_contracts
if [ $# -eq 1 ];
then
    start_polkadot_launch $1
else
    start_polkadot_launch
fi
echo "Waiting for consensus between polkadot and parachain"
sleep 60
start_relayer

echo "Process Tree:"
pstree $$

sleep 3
until $(grep "Syncing headers starting..." $(pwd)/ethereum-relay.log > /dev/null); do
    echo "Waiting for relayer to generate the DAG cache. This can take up to 20 minutes."
    sleep 20
done

until $(grep "Done retrieving finalized headers" $(pwd)/ethereum-relay.log > /dev/null); do
    echo "Waiting for relayer to sync headers..."
    sleep 5
done

echo "System has been initialized"

wait
