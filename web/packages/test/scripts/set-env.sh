root_dir="$(realpath ../../..)"
web_dir="$root_dir/web"
lodestar_version="${LODESTAR_VER:-1.8.0}"
export contract_dir="$root_dir/contracts"
test_helpers_dir="$web_dir/packages/test-helpers"
relay_dir="$root_dir/relayer"
relay_bin="$relay_dir/build/snowbridge-relay"
export output_dir="${OUTPUT_DIR:-/tmp/snowbridge}"
export output_bin_dir="$output_dir/bin"
ethereum_data_dir="$output_dir/ethereum"
zombienet_data_dir="$output_dir/zombienet"
export PATH="$output_bin_dir:$PATH"

active_spec="${ACTIVE_SPEC:-minimal}"
eth_network="${ETH_NETWORK:-localhost}"
eth_endpoint_http="${ETH_RPC_ENDPOINT:-http://127.0.0.1:8545}/${INFURA_PROJECT_ID:-}"
eth_endpoint_ws="${ETH_WS_ENDPOINT:-ws://127.0.0.1:8546}/${INFURA_PROJECT_ID:-}"
eth_gas_limit="${ETH_GAS_LIMIT:-5000000}"
eth_chain_id="${ETH_NETWORK_ID:-15}"
eth_fast_mode="${ETH_FAST_MODE:-false}"
etherscan_api_key="${ETHERSCAN_API_KEY:-}"

parachain_relay_eth_key="${PARACHAIN_RELAY_ETH_KEY:-0x8013383de6e5a891e7754ae1ef5a21e7661f1fe67cd47ca8ebf4acd6de66879a}"
beefy_relay_eth_key="${BEEFY_RELAY_ETH_KEY:-0x935b65c833ced92c43ef9de6bff30703d941bd92a2637cb00cfad389f5862109}"

# Parachain accounts for which the relayer will relay messages over the basic channel.
# These IDs are for the test accounts Alice, Bob, Charlie, Dave, Eve and Ferdie, in order
basic_parachain_account_ids="${BASIC_PARACHAIN_ACCOUNT_IDS:-0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d,0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48,0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22,0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20,0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e,0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c}"
# Ethereum addresses for which the relayer will relay messages over the basic channel.
# This address is for the default eth account used in the E2E tests, taken from test/src/ethclient/index.js.
basic_eth_addresses="${BASIC_ETH_ADDRESSES:-0x89b4ab1ef20763630df9743acf155865600daff2}"
beacon_endpoint_http="${BEACON_HTTP_ENDPOINT:-http://127.0.0.1:9596}"

# Local substrate chain endpoints
bridgehub_ws_url="${BRIDGE_HUB_WS_URL:-ws://127.0.0.1:11144}"
bridgehub_seed="${BRIDGE_HUB_SEED:-//Alice}"
bridgehub_pallets_owner="${BRIDGE_HUB_PALLETS_OWNER:-0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d}"
export BRIDGE_HUB_PARAID="${BRIDGE_HUB_PARAID:-1013}"
export BRIDGE_HUB_AGENT_ID="${BRIDGE_HUB_AGENT_ID:-0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314}"

assethub_ws_url="${ASSET_HUB_WS_URL:-ws://127.0.0.1:12144}"
assethub_seed="${ASSET_HUB_SEED:-//Alice}"
export ASSET_HUB_PARAID="${ASSET_HUB_PARAID:-1000}"
export ASSET_HUB_AGENT_ID="${ASSET_HUB_AGENT_ID:-0x72456f48efed08af20e5b317abf8648ac66e86bb90a411d9b0b713f7364b75b4}"
export TEMPLATE_PARA_ID="${TEMPLATE_PARA_ID:-1001}"
export TEMPLATE_AGENT_ID="${TEMPLATE_AGENT_ID:-0x2075b9f5bc236462eb1473c9a6236c3588e33ed19ead53aa3d9c62ed941cb793}"

export ASSET_HUB_CHANNEL_ID="0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539"
export TEMPLATE_CHANNEL_ID="0x26c13363ad6499b895574b3ca482545dda41d657ffc5673b39a218cd34053e5b"
export PRIMARY_GOVERNANCE_CHANNEL_ID="0x0000000000000000000000000000000000000000000000000000000000000001"
export SECONDARY_GOVERNANCE_CHANNEL_ID="0x0000000000000000000000000000000000000000000000000000000000000002"

# ROC and KSM amounts need to be multiplied by 10^(18 - 12) to have the same number
# of decimal places as ETH (18 decimal places)
export DOT_TO_ETH_DECIMALS=1000000

relaychain_ws_url="${RELAYCHAIN_WS_URL:-ws://127.0.0.1:9944}"
relaychain_sudo_seed="${RELAYCHAIN_SUDO_SEED:-//Alice}"

skip_relayer="${SKIP_RELAYER:-false}"

## Important accounts

# Account for assethub (Sibling parachain 1000 5Eg2fntNprdN3FgH4sfEaaZhYtddZQSQUqvYJ1f2mLtinVhV in testnet)
assethub_sovereign_account="${ASSETHUB_SOVEREIGN_ACCOUNT:-0x7369626ce8030000000000000000000000000000000000000000000000000000}"
# Account for template (Sibling parachain 1001 5Eg2fntP2UgYZNc5tW8xmmCmeXJ3hmNXaZ9MvcpbMdvh1bBJ in testnet)
template_sovereign_account="${TEMPLATE_SOVEREIGN_ACCOUNT:-0x7369626ce9030000000000000000000000000000000000000000000000000000}"
# Beacon relay account (//BeaconRelay 5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c in testnet)
beacon_relayer_pub_key="${BEACON_RELAYER_PUB_KEY:-0xc46e141b5083721ad5f5056ba1cded69dce4a65f027ed3362357605b1687986a}"
# Execution relay account (//ExecutionRelay 5CFNWKMFPsw5Cs2Teo6Pvg7rWyjKiFfqPZs8U4MZXzMYFwXL in testnet)
execution_relayer_pub_key="${EXECUTION_RELAYER_PUB_KEY:-0x08228efd065c58a043da95c8bf177659fc587643e71e7ed1534666177730196f}"

# Config for deploying contracts

## Deployment key
export PRIVATE_KEY="${DEPLOYER_ETH_KEY:-0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77}"

## BeefyClient
# For max safety delay should be MAX_SEED_LOOKAHEAD=4 epochs=4*8*6=192s
# but for rococo-local each session is only 20 slots=120s
# so relax somehow here just for quick test
# for production deployment ETH_RANDAO_DELAY should be configured in a more reasonable sense
export RANDAO_COMMIT_DELAY="${ETH_RANDAO_DELAY:-3}"
export RANDAO_COMMIT_EXP="${ETH_RANDAO_EXP:-3}"
export MINIMUM_REQUIRED_SIGNATURES="${MINIMUM_REQUIRED_SIGNATURES:-16}"

export REJECT_OUTBOUND_MESSAGES=false
export DEFAULT_FEE="${ETH_DEFAULT_FEE:-1}"

export CREATE_CALL_INDEX="${ETH_CREATE_CALL_INDEX:-0x3500}"

export REGISTER_NATIVE_TOKEN_FEE="${ETH_REGISTER_NATIVE_TOKEN_FEE:-0}"
export SEND_NATIVE_TOKEN_FEE="${ETH_SEND_NATIVE_TOKEN_FEE:-0}"

## Vault
export BRIDGE_HUB_INITIAL_DEPOSIT="${ETH_BRIDGE_HUB_INITIAL_DEPOSIT:-10000000000000000000}"


export GATEWAY_PROXY_CONTRACT="${GATEWAY_PROXY_CONTRACT:-0xEDa338E4dC46038493b885327842fD3E301CaB39}"

address_for() {
    jq -r ".contracts.${1}.address" "$output_dir/contracts.json"
}

kill_all() {
    trap - SIGTERM
    kill 0
}

cleanup() {
    echo "Cleaning resource"
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

wait_contract_deployed() {
    local ready=""
    while [ -z "$ready" ]; do
        if [ -f "$output_dir/contracts.json" ]; then
            ready="true"
        fi
        sleep 2
    done
}
