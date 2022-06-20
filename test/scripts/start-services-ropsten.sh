#!/usr/bin/env bash
set -eu

root_dir="$(realpath ..)"
parachain_dir="$root_dir/parachain"
ethereum_dir="$root_dir/ethereum"
relay_dir="$root_dir/relayer"

infura_endpoint_http="https://ropsten.infura.io/v3/$INFURA_PROJECT_ID"
infura_endpoint_ws="wss://ropsten.infura.io/ws/v3/$INFURA_PROJECT_ID"

output_dir=/tmp/snowbridge

address_for()
{
    jq -r ".contracts.${1}.address" "$output_dir/contracts.json"
}

deploy_contracts()
{
    echo "Deploying contracts"
    (
        cd $ethereum_dir
        npx hardhat deploy --network ropsten --reset --export "$output_dir/contracts.json"
    )

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

start_polkadot_launch()
{
    if [[ -z "${POLKADOT_BIN+x}" ]]; then
        echo "Please specify the path to the polkadot binary. Variable POLKADOT_BIN is unset."
    fi

    local parachain_bin="$parachain_dir/target/release/snowbridge"
    local test_collator_bin="$parachain_dir/utils/test-parachain/target/release/snowbridge-test-collator"

    echo "Building snowbridge parachain"
    cargo build \
        --manifest-path "$parachain_dir/Cargo.toml" \
        --release \
        --no-default-features \
        --features snowbase-native,rococo-native

    echo "Building query tool"
    cargo build --release --manifest-path "$parachain_dir/tools/query-events/Cargo.toml"

    cp "$parachain_dir/target/release/snowbridge-query-events" "$output_dir/bin"

    echo "Building test parachain"
    cargo build --manifest-path "$parachain_dir/utils/test-parachain/Cargo.toml" --release

    echo "Generating chain specification"
    "$parachain_bin" build-spec --chain snowbase --disable-default-bootnode > "$output_dir/spec.json"

    echo "Updating chain specification"
    curl $infura_endpoint_http \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
        | node scripts/helpers/transformEthHeader.js > "$output_dir/initialHeader.json"

    cat "$output_dir/spec.json" | node scripts/helpers/mutateSpec.js "$output_dir/initialHeader.json" "$output_dir/contracts.json" | sponge "$output_dir/spec.json"

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

    RELAYCHAIN_ENDPOINT="ws://localhost:9944" npx hardhat run ./scripts/configure-beefy.ts --network ropsten

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

    local relay_bin="$relay_dir/build/snowbridge-relay"

    # Launch beefy relay
    (
        : > beefy-relay.log
        while :
        do
            echo "Starting beefy relay at $(date)"
            "${relay_bin}" run beefy \
                --config "$output_dir/beefy-relay.json" \
                --ethereum.private-key "$BEEFY_RELAY_ETH_KEY" \
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
                --ethereum.private-key "$PARACHAIN_RELAY_ETH_KEY" \
                >>parachain-relay.log 2>&1 || true
            sleep 20
        done
    ) &

    # Launch ethereum relay
    (
        : > ethereum-relay.log
        while :
        do
          echo "Starting ethereum relay at $(date)"
            "${relay_bin}" run ethereum \
                --config $output_dir/ethereum-relay.json \
                --substrate.private-key "//Relay" \
                >>ethereum-relay.log 2>&1 || true
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

deploy_contracts
start_polkadot_launch

echo "Waiting for consensus between polkadot and parachain"
sleep 60
configure_contracts
start_relayer

echo "Process Tree:"
pstree -T $$

echo "Testnet has been initialized"

wait
