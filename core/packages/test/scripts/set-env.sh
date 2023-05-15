#!/usr/bin/env sh

# exports enable use in core/packages/test/config/launch-config.toml
root_dir="$(realpath ../../..)"
bridge_hub_runtime="${PARACHAIN_RUNTIME:-bridge-hub-rococo-local}"
relaychain_version="${POLKADOT_VER:-v0.9.38}"
relaychain_dir="$root_dir/parachain/.cargo/$relaychain_version"
relaychain_bin="${POLKADOT_BIN:-$relaychain_dir/bin/polkadot}"
cumulus_version="${CUMULUS_VER:-snowbridge}"
cumulus_dir="$root_dir/parachain/.cargo/$cumulus_version"
cumulus_bin="${CUMULUS_BIN:-$cumulus_dir/bin/polkadot-parachain}"
core_dir="$root_dir/core"
lodestar_version="${LODESTAR_VER:-1.5.1}"
geth_version="${GETH_VER:-v1.11.2}"
geth_dir="$root_dir/../go-ethereum/$geth_version"
contract_dir="$core_dir/packages/contracts"
relay_dir="$root_dir/relayer"
relay_bin="$relay_dir/build/snowbridge-relay"
export output_dir=/tmp/snowbridge
export output_bin_dir="$output_dir/bin"
ethereum_data_dir="$output_dir/geth"
export PATH="$output_bin_dir:$PATH"

# Because we can run a local node for public network like goerli or mainnet, add active_spec as a separate config here
# to decouple with eth_network, explicit set ACTIVE_SPEC to mainnet when test with public network
active_spec="${ACTIVE_SPEC:-minimal}"
eth_network="${ETH_NETWORK:-localhost}"
eth_endpoint_http="${ETH_RPC_ENDPOINT:-http://127.0.0.1:8545}/${INFURA_PROJECT_ID:-}"
eth_endpoint_ws="${ETH_WS_ENDPOINT:-ws://127.0.0.1:8546}/${INFURA_PROJECT_ID:-}"
eth_gas_limit="${ETH_GAS_LIMIT:-5000000}"

beefy_state_file="${BEEFY_STATE_FILE:-$output_dir/beefy-state.json}"
beefy_start_block="${BEEFY_START_BLOCK:-12}"

parachain_relay_eth_key="${PARACHAIN_RELAY_ETH_KEY:-0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a}"
beefy_relay_eth_key="${BEEFY_RELAY_ETH_KEY:-0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109}"

# Parachain accounts for which the relayer will relay messages over the basic channel.
# These IDs are for the test accounts Alice, Bob, Charlie, Dave, Eve and Ferdie, in order
basic_parachain_account_ids="${BASIC_PARACHAIN_ACCOUNT_IDS:-0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d,0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48,0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22,0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20,0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e,0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c}"
# Ethereum addresses for which the relayer will relay messages over the basic channel.
# This address is for the default eth account used in the E2E tests, taken from test/src/ethclient/index.js.
basic_eth_addresses="${BASIC_ETH_ADDRESSES:-0x89b4ab1ef20763630df9743acf155865600daff2}"
beacon_endpoint_http="${BEACON_HTTP_ENDPOINT:-http://127.0.0.1:9596}"

# Config for deploying contracts

## Deployment key
export PRIVATE_KEY="${DEPLOYER_ETH_KEY:-0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77}"

## BeefyClient
export RANDAO_COMMIT_DELAY=3
export RANDAO_COMMIT_EXP=8

## ParachainClient
export BRIDGE_HUB_PARAID=1001

## OutboundChannel
export RELAYER_FEE=100

## InboundChannel
export RELAYER_REWARD=100

## NativeTokens
export ASSET_HUB_PARAID=1000
export CREATE_TOKEN_FEE="1000000000000000000"

address_for()
{
    jq -r ".contracts.${1}.address" "$output_dir/contracts.json"
}

kill_all() {
    trap - SIGTERM
    kill 0
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
}
