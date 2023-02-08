root_dir="$(realpath ../../..)"
relaychain_dir="${POLKADOT_DIR:-$root_dir/relaychain}"
relaychain_version="${POLKADOT_VER:-v0.9.30}"
relaychain_bin="${POLKADOT_BIN:-$relaychain_dir/target/release/polkadot}"
parachain_dir="$root_dir/parachain"
parachain_runtime="${PARACHAIN_RUNTIME:-snowbase}"
parachain_bin="$parachain_dir/target/release/snowbridge"
test_collator_bin="$parachain_dir/utils/test-parachain/target/release/snowbridge-test-node"
core_dir="$root_dir/core"
lodestar_version="${LODESTAR_VER:-1.4.2}"
contract_dir="$core_dir/packages/contracts"
relay_dir="$root_dir/relayer"
relay_bin="$relay_dir/build/snowbridge-relay"
[ "$parachain_runtime" = "snowbase" ] &&
   seconds_per_slot=6 ||
   seconds_per_slot=12
export output_dir=/tmp/snowbridge
export output_bin_dir="$output_dir/bin"
ethereum_data_dir="$output_dir/geth"
export PATH="$output_bin_dir:$PATH"

eth_network="${ETH_NETWORK:-localhost}"
infura_endpoint_http="${ETH_RPC_ENDPOINT:-http://localhost:8545}/${INFURA_PROJECT_ID:-}"
infura_endpoint_ws="${ETH_WS_ENDPOINT:-ws://localhost:8546}/${INFURA_PROJECT_ID:-}"

parachain_relay_eth_key="${PARACHAIN_RELAY_ETH_KEY:-0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a}"
beefy_relay_eth_key="${BEEFY_RELAY_ETH_KEY:-0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109}"

# Parachain accounts for which the relayer will relay messages over the basic channel.
# These IDs are for the test accounts Alice, Bob, Charlie, Dave, Eve and Ferdie, in order
basic_parachain_account_ids="${BASIC_PARACHAIN_ACCOUNT_IDS:-0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d,0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48,0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22,0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20,0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e,0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c}"
# Ethereum addresses for which the relayer will relay messages over the basic channel.
# This address is for the default eth account used in the E2E tests, taken from test/src/ethclient/index.js.
basic_eth_addresses="${BASIC_ETH_ADDRESSES:-0x89b4ab1ef20763630df9743acf155865600daff2}"
beacon_endpoint_http="${BEACON_HTTP_ENDPOINT:-http://localhost:9596}"


address_for()
{
    jq -r ".contracts.${1}.address" "$output_dir/contracts.json"
}

kill_trap() {
    trap - SIGTERM
    pkill -P $$
}

kill_all() {
    kill_trap
    kill_chains
    kill_relayer
}

kill_chains() {
    echo "Killing chains"
    kill_polkadot
    kill_ethereum
}

kill_ethereum() {
    pkill -9 -f lodestar
    pkill -9 geth
}

kill_polkadot() {
    pkill -9 polkadot
    pkill -9 snowbridge-test-node
    pkill -9 snowbridge
    pkill -9 -f polkadot-launch
    pkill -9 zombienet
}

kill_relayer() {
    echo "Killing relayer"
    kill_trap
    sleep 3
    pkill -9 snowbridge-relay
    sleep 1
}

cleanup() {
    echo "Cleaning resource"
    rm -rf *.log
    rm -rf "$output_dir"
    mkdir "$output_dir"
    mkdir "$output_bin_dir"
    mkdir "$ethereum_data_dir"
}

check_tool() {
    if ! [ -x "$(command -v g++)" ]; then
        echo 'Error: g++ is not installed.'
        exit
    fi
    if ! [ -x "$(command -v protoc)" ]; then
        echo 'Error: protoc is not installed.'
        exit
    fi
    if ! [ -x "$(command -v jq)" ]; then
        echo 'Error: jq is not installed.'
        exit
    fi
    if ! [ -x "$(command -v geth)" ]; then
        echo 'Error: geth is not installed.'
        exit
    fi
    if ! [ -x "$(command -v sponge)" ]; then
        echo 'Error: sponge is not installed.'
        exit
    fi
    if ! [ -x "$(command -v direnv)" ]; then
        echo 'Error: direnv is not installed.'
        exit
    fi
    if ! [ -x "$(command -v mage)" ]; then
        echo 'Error: mage is not installed.'
        exit
    fi
    if ! [ -x "$(command -v pnpm)" ]; then
        echo 'Error: pnpm is not installed.'
        exit
    fi
    if ! [ -x "$(command -v zombienet)" ]; then
        echo 'Error: zombienet is not installed.'
        exit
    fi
    if [[ "$OSTYPE" =~ ^darwin ]]; then
        if ! [ -x "$(command -v gdate)" ]; then
            echo 'Error: gdate is not installed.'
            exit
        fi
        if ! [ -x "$(command -v gsed)" ]; then
            echo 'Error: gsed is not installed.'
            exit
        fi
    fi
}

