#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

configure_beefy()
{
    pushd "$contract_dir"

    npx ts-node ./scripts/configure-beefy.ts

    current_id=$(jq .validatorSets.current.id $beefy_state_file)
    current_length=$(jq .validatorSets.current.length $beefy_state_file)
    current_root=$(jq .validatorSets.current.root $beefy_state_file)

    next_id=$(jq .validatorSets.next.id $beefy_state_file)
    next_length=$(jq .validatorSets.next.length $beefy_state_file)
    next_root=$(jq .validatorSets.next.root $beefy_state_file)

    # remove double quote before cast
    current_root=$(sed -e 's/^"//' -e 's/"$//' <<< $current_root)
    next_root=$(sed -e 's/^"//' -e 's/"$//' <<< $next_root)

    echo "Transact call to initialize BeefyClient"
    # sometimes sending transact with cast will fail on the goerli network
    # temporarily resolved by manually setting --gas-price and --gas-limit
    cast send $(address_for BeefyClient) \
    	"initialize(uint64,(uint128,uint128,bytes32),(uint128,uint128,bytes32))" \
    	--rpc-url $eth_endpoint_http \
    	--private-key $PRIVATE_KEY \
      $beefy_start_block \
      \($current_id,$current_length,$current_root\) \
      \($next_id,$next_length,$next_root\) || true # "|| true" can be removed once https://github.com/foundry-rs/foundry/pull/4010 has been released
    popd
}

if [ -z "${from_start_services:-}" ]; then
    echo "config contracts only!"
    configure_beefy
    wait
fi
