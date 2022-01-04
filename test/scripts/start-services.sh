#!/usr/bin/env bash
set -eu

root_dir="$(realpath ..)"
parachain_dir="$root_dir/parachain"
ethereum_dir="$root_dir/ethereum"
relay_dir="$root_dir/relayer"

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

    geth init --datadir "$data_dir" config/genesis.json
    geth account import --datadir "$data_dir" --password /dev/null config/dev-example-key0.prv
    geth account import --datadir "$data_dir" --password /dev/null config/dev-example-key1.prv
    geth --vmdebug --datadir "$data_dir" --networkid 15 \
        --http --http.api debug,personal,eth,net,web3,txpool --ws --ws.api debug,eth,net,web3 \
        --rpc.allow-unprotected-txs --mine --miner.threads=1 \
        --miner.etherbase=0x0000000000000000000000000000000000000000 \
        --allow-insecure-unlock \
        --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
        --password /dev/null \
        --rpc.gascap 100000000 \
        --trace "$data_dir/trace" \
        --gcmode archive \
        --miner.gasprice=0 \
        > "$output_dir/geth.log" 2>&1 &
}

deploy_contracts()
{
    echo "Deploying contracts"
    (
        cd $ethereum_dir
        npx hardhat deploy --network localhost --reset --export "$output_dir/contracts.json"
    )

    echo "Exported contract artifacts: $output_dir/contracts.json"
}

start_polkadot_launch()
{
    if [[ -z "${POLKADOT_BIN+x}" ]]; then
        echo "Please specify the path to the polkadot binary. Variable POLKADOT_BIN is unset."
    fi

    local parachain_bin="$parachain_dir/target/release/snowbridge"
    local test_collator_bin="$parachain_dir/target/release/snowbridge-test-collator"

    echo "Building parachain node"
    cargo build --workspace \
        --manifest-path "$parachain_dir/Cargo.toml" \
        --release \
        --no-default-features \
        --features with-local-runtime

    echo "Generating chain specification"
    "$parachain_bin" build-spec --disable-default-bootnode > "$output_dir/snowbridge_spec.json"

    echo "Updating chain specification with ethereum state"
    header=$(curl http://localhost:8545 \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
        | node ../test/scripts/helpers/transformEthHeader.js)

    jq \
        --argjson header "$header" \
        ' .genesis.runtime.ethereumLightClient.initialHeader = $header
        | .genesis.runtime.ethereumLightClient.initialDifficulty = "0x0"
        | .genesis.runtime.parachainInfo.parachainId = 1000
        | .para_id = 1000
        ' \
        "$output_dir/snowbridge_spec.json" | sponge "$output_dir/snowbridge_spec.json"

    if [[ -n "${TEST_MALICIOUS_APP+x}" ]]; then
        jq '.genesis.runtime.dotApp.address = "0x433488cec14C4478e5ff18DDC7E7384Fc416f148"' \
        "$output_dir/snowbridge_spec.json" | sponge "$output_dir/snowbridge_spec.json"
    fi

    echo "Generating test chain specification"
    "$test_collator_bin" build-spec --disable-default-bootnode > "$output_dir/snowbridge_test_spec.json"

    echo "Updating test chain specification"
    jq \
        ' .genesis.runtime.parachainInfo.parachainId = 1001
        | .para_id = 1001
        ' \
        "$output_dir/snowbridge_test_spec.json" | sponge "$output_dir/snowbridge_test_spec.json"

    jq \
        --arg polkadot "$(realpath $POLKADOT_BIN)" \
        --arg bin "$parachain_bin" \
        --arg spec "$output_dir/snowbridge_spec.json" \
        --arg test_collator "$(realpath $test_collator_bin)" \
        --arg test_spec "$output_dir/snowbridge_test_spec.json" \
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

    RELAYCHAIN_ENDPOINT="ws://localhost:9944" npx hardhat run ./scripts/configure-beefy.ts --network localhost

    popd
}

start_relayer()
{
    echo "Starting relay services"

    # Build relay services
    mage -d ../relayer build

    # Configure beefy relay
    jq \
        --arg k1 "$(address_for BeefyLightClient)" \
    '
      .sink.contracts.BeefyLightClient = $k1
    ' \
    config/beefy-relay.json > $output_dir/beefy-relay.json

    # Configure parachain relay
    jq \
        --arg k1 "$(address_for BasicInboundChannel)" \
        --arg k2 "$(address_for IncentivizedInboundChannel)" \
        --arg k3 "$(address_for BeefyLightClient)" \
    '
      .source.contracts.BasicInboundChannel = $k1
    | .source.contracts.IncentivizedInboundChannel = $k2
    | .source.contracts.BeefyLightClient = $k3
    | .sink.contracts.BasicInboundChannel = $k1
    | .sink.contracts.IncentivizedInboundChannel = $k2
    ' \
    config/parachain-relay.json > $output_dir/parachain-relay.json

    # Configure ethereum relay
    jq \
        --arg k1 "$(address_for BasicOutboundChannel)" \
        --arg k2 "$(address_for IncentivizedOutboundChannel)" \
    '
      .source.contracts.BasicOutboundChannel = $k1
    | .source.contracts.IncentivizedOutboundChannel = $k2
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
                --ethereum.private-key "0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109" \
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
                --ethereum.private-key "0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a" \
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

if [[ -f ".env" ]]; then
    export $(<.env)
fi

rm -rf "$output_dir"
mkdir "$output_dir"

start_geth
deploy_contracts
start_polkadot_launch

echo "Waiting for consensus between polkadot and parachain"
sleep 60
configure_contracts
start_relayer

echo "Process Tree:"
pstree -T $$

sleep 3
until grep "Syncing headers starting..." ethereum-relay.log > /dev/null; do
    echo "Waiting for ethereum relay to generate the DAG cache. This can take up to 20 minutes."
    sleep 20
done

until grep "Done retrieving finalized headers" ethereum-relay.log > /dev/null; do
    echo "Waiting for ethereum relay to sync headers..."
    sleep 5
done


echo "Testnet has been initialized"

wait
