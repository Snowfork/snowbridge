#!/usr/bin/env bash
set -eu

root_dir="$(realpath ..)"
parachain_dir="$root_dir/parachain"
ethereum_dir="$root_dir/ethereum"
relay_dir="$root_dir/relayer"

eth_network="${ETH_NETWORK:-localhost}"
infura_endpoint_http="${ETH_RPC_ENDPOINT:-http://localhost:8545}/${INFURA_PROJECT_ID:-}"
infura_endpoint_ws="${ETH_WS_ENDPOINT:-ws://localhost:8546}/${INFURA_PROJECT_ID:-}"

parachain_relay_eth_key="${PARACHAIN_RELAY_ETH_KEY:-0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a}"
beefy_relay_eth_key="${BEEFY_RELAY_ETH_KEY:-0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109}"

beacon_endpoint_http="${BEACON_HTTP_ENDPOINT:-http://localhost:9596}"

output_dir=/tmp/snowbridge

address_for()
{
    jq -r ".contracts.${1}.address" "$output_dir/contracts.json"
}

start_geth() {
    if [[ -n "${DIFFICULTY+x}" ]]; then
        jq --arg difficulty "${DIFFICULTY}" \
            '.difficulty = $difficulty' \
            config/genesis.json \
            > "$output_dir/genesis.json"
    else
        cp config/genesis.json "$output_dir/genesis.json"
    fi

    local data_dir="$output_dir/geth"
    
    if [ "$eth_network" == "localhost" ]; then
        echo "Starting geth local net"

        geth init --datadir "$data_dir" "$output_dir/genesis.json"
        geth account import --datadir "$data_dir" --password /dev/null config/dev-example-key0.prv
        geth account import --datadir "$data_dir" --password /dev/null config/dev-example-key1.prv
        geth --vmdebug --datadir "$data_dir" --networkid 15 \
            --http --http.api debug,personal,eth,net,web3,txpool,engine,miner --ws --ws.api debug,eth,net,web3 \
            --rpc.allow-unprotected-txs --mine --miner.threads=1 \
            --miner.etherbase=0x0000000000000000000000000000000000000000 \
            --allow-insecure-unlock \
            --authrpc.jwtsecret config/jwtsecret \
            --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
            --password /dev/null \
            --rpc.gascap 100000000 \
            --trace "$data_dir/trace" \
            --gcmode archive \
            --miner.gasprice=0 \
            > "$output_dir/geth.log" 2>&1 &
    fi
}

start_lodestar() {
    if [ "$eth_network" == "localhost" ]; then
        echo "Waiting for geth API to be ready"
        sleep 2

        genesisHash=$(curl http://localhost:8545 \
            -X POST \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')

        timestamp=$(date -d'+10second' +%s)

        echo "Starting lodestar local net"

        lodestar dev \
            --genesisValidators 8 \
            --genesisTime $timestamp \
            --startValidators "0..7" \
            --enr.ip "127.0.0.1" \
            --rootDir "$output_dir/beacon-$timestamp" \
            --reset \
            --terminal-total-difficulty-override 0 \
            --genesisEth1Hash $genesisHash \
            --params.ALTAIR_FORK_EPOCH 0 \
            --params.BELLATRIX_FORK_EPOCH 0 \
            --eth1.enabled=true \
            --api.rest.api="beacon,config,events,node,validator,lightclient" \
            --jwt-secret config/jwtsecret \
            > "$output_dir/lodestar.log" 2>&1 &
    fi

    echo "Started up beacon node"
}

deploy_contracts()
{
    echo "Deploying contracts"
    (
        cd $ethereum_dir
        npx hardhat deploy --network $eth_network --reset --export "$output_dir/contracts.json"
    )

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

start_polkadot_launch()
{
    if [[ -z "${POLKADOT_BIN+x}" ]]; then
        echo "Please specify the path to the polkadot binary. Variable POLKADOT_BIN is unset."
    fi

    local parachain_bin="$parachain_dir/target/release/snowbridge"
    local test_collator_bin="$parachain_dir/utils/test-parachain/target/release/snowbridge-test-node"

    runtime="snowbase"

    if [ "$eth_network" != "localhost" ]; then
        runtime="snowblink"
    fi

    echo "Building snowbridge parachain"
    cargo build \
        --manifest-path "$parachain_dir/Cargo.toml" \
        --release \
        --no-default-features \
        --features "${runtime}-native,rococo-native" \
        --bin snowbridge

    echo "Building query tool"
    cargo build --release --manifest-path "$parachain_dir/tools/query-events/Cargo.toml" --bin snowbridge-query-events

    cp "$parachain_dir/target/release/snowbridge-query-events" "$output_dir/bin"

    echo "Building test parachain"
    cargo build --manifest-path "$parachain_dir/utils/test-parachain/Cargo.toml" --release --bin snowbridge-test-node

    echo "Generating chain specification"
    "$parachain_bin" build-spec --chain "$runtime" --disable-default-bootnode > "$output_dir/spec.json"

    echo "Updating chain specification"
    curl $infura_endpoint_http \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
        | node scripts/helpers/transformEthHeader.js > "$output_dir/initialHeader.json"

    initial_beacon_block=$(curl "$beacon_endpoint_http/eth/v1/beacon/states/head/finality_checkpoints" \
            | jq -r '.data.finalized.root')

    curl "$beacon_endpoint_http/eth/v1/light_client/bootstrap/$initial_beacon_block" \
        | node scripts/helpers/transformInitialBeaconSync.js > "$output_dir/initialBeaconSync_tmp.json"

    validatorsRoot=$(curl "$beacon_endpoint_http/eth/v1/beacon/genesis" \
            | jq -r '.data.genesis_validators_root')

    jq \
        --arg validatorsRoot "$validatorsRoot" \
        ' .validators_root = $validatorsRoot
        ' \
        "$output_dir/initialBeaconSync_tmp.json" \
        > "$output_dir/initialBeaconSync.json"

    cat "$output_dir/spec.json" | node scripts/helpers/mutateSpec.js "$output_dir/initialHeader.json" "$output_dir/contracts.json" "$output_dir/initialBeaconSync.json" | sponge "$output_dir/spec.json"

    # TODO: add back
    # if [[ -n "${TEST_MALICIOUS_APP+x}" ]]; then
    #     jq '.genesis.runtime.dotApp.address = "0x433488cec14C4478e5ff18DDC7E7384Fc416f148"' \
    #     "$output_dir/spec.json" | sponge "$output_dir/spec.json"
    # fi

    echo "Generating test chain specification"
    "$test_collator_bin" build-spec --disable-default-bootnode > "$output_dir/test_spec.json"

    echo "Updating test chain specification"
    jq \
        ' .genesis.runtime.parachainInfo.parachainId = 1001
        | .para_id = 1001
        ' \
        "$output_dir/test_spec.json" | sponge "$output_dir/test_spec.json"

    jq \
        --arg polkadot "$(realpath $POLKADOT_BIN)" \
        --arg bin "$parachain_bin" \
        --arg spec "$output_dir/spec.json" \
        --arg test_collator "$(realpath $test_collator_bin)" \
        --arg test_spec "$output_dir/test_spec.json" \
        ' .relaychain.bin = $polkadot
        | .parachains[0].bin = $bin
        | .parachains[0].chain = $spec
        | .parachains[1].bin = $test_collator
        | .parachains[1].chain = $test_spec
        ' \
        config/launch-config.json \
        > "$output_dir/launch-config.json"

    polkadot-launch "$output_dir/launch-config.json" &

    scripts/wait-for-it.sh -t 120 localhost:11144
    scripts/wait-for-it.sh -t 120 localhost:13144
}

configure_contracts()
{
    echo "Configuring contracts"
    pushd ../ethereum

    RELAYCHAIN_ENDPOINT="ws://localhost:9944" npx hardhat run ./scripts/configure-beefy.ts --network $eth_network

    popd
}

start_relayer()
{
    echo "Starting relay services"

    # Build relay services
    mage -d ../relayer build

    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyClient)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
    '
      .sink.contracts.BeefyClient = $k1
    | .source.ethereum.endpoint = $infura_endpoint_ws
    | .sink.ethereum.endpoint = $infura_endpoint_ws
    ' \
    config/beefy-relay.json > $output_dir/beefy-relay.json

    # Configure parachain relay
    jq \
        --arg k1 "$(address_for BasicInboundChannel)" \
        --arg k2 "$(address_for IncentivizedInboundChannel)" \
        --arg k3 "$(address_for BeefyClient)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
    '
      .source.contracts.BasicInboundChannel = $k1
    | .source.contracts.IncentivizedInboundChannel = $k2
    | .source.contracts.BeefyClient = $k3
    | .sink.contracts.BasicInboundChannel = $k1
    | .sink.contracts.IncentivizedInboundChannel = $k2
    | .source.ethereum.endpoint = $infura_endpoint_ws
    | .sink.ethereum.endpoint = $infura_endpoint_ws
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay.json

    # Configure ethereum relay
    jq \
        --arg k1 "$(address_for BasicOutboundChannel)" \
        --arg k2 "$(address_for IncentivizedOutboundChannel)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
    '
      .source.contracts.BasicOutboundChannel = $k1
    | .source.contracts.IncentivizedOutboundChannel = $k2
    | .source.ethereum.endpoint = $infura_endpoint_ws
    ' \
    config/ethereum-relay.json > $output_dir/ethereum-relay.json

    active_spec="mainnet"
    if [ "$eth_network" == "localhost" ]; then
       active_spec="minimal"
    fi

    # Configure beacon relay
    jq \
        --arg k1 "$(address_for BasicOutboundChannel)" \
        --arg k2 "$(address_for IncentivizedOutboundChannel)" \
        --arg infura_endpoint_ws $infura_endpoint_ws \
        --arg beacon_endpoint_http $beacon_endpoint_http \
        --arg active_spec $active_spec \
    '
      .source.contracts.BasicOutboundChannel = $k1
    | .source.contracts.IncentivizedOutboundChannel = $k2
    | .source.ethereum.endpoint = $infura_endpoint_ws
    | .source.beacon.endpoint = $beacon_endpoint_http
    | .source.beacon.activeSpec = $active_spec
    ' \
    config/beacon-relay.json > $output_dir/beacon-relay.json

    local relay_bin="$relay_dir/build/snowbridge-relay"

    # Launch beefy relay
    (
        : > beefy-relay.log
        while :
        do
            echo "Starting beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key $beefy_relay_eth_key \
                >>beefy-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch parachain relay
    (
        : > parachain-relay.log
        while :
        do
          echo "Starting parachain relay at $(date)"
            "${relay_bin}" run parachain \
                --config "$output_dir/parachain-relay.json" \
                --ethereum.private-key $parachain_relay_eth_key \
                >>parachain-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch beacon relay
    (
        : > beacon-relay.log
        while :
        do
        echo "Starting beacon relay at $(date)"
            "${relay_bin}" run beacon \
                --config $output_dir/beacon-relay.json \
                --substrate.private-key "//BeaconRelay" \
                >>beacon-relay.log 2>&1 || true
            sleep 20
        done
    ) &
}

cleanup() {
    trap - SIGTERM
    kill -- -"$(ps -o pgid:1= $$)"
}

trap cleanup SIGINT SIGTERM EXIT

rm -rf "$output_dir"
mkdir "$output_dir"
mkdir "$output_dir/bin"

export PATH="$output_dir/bin:$PATH"

start_geth
start_lodestar

deploy_contracts
start_polkadot_launch

echo "Waiting for consensus between polkadot and parachain"
sleep 60
configure_contracts
start_relayer

echo "Process Tree:"
pstree -T $$

until grep "starting to sync finalized headers" beacon-relay.log > /dev/null; do
    echo "Waiting for beacon relay to sync headers..."
    sleep 5
done

echo "Testnet has been initialized"

wait
