#!/usr/bin/env bash
set -eu


generate_beefy_fixture() {
    num="$1"
    # Generate validator set
    pnpm generateBeefyValidatorSet

    # Generate bit field
    pushd ../../../contracts
    forge build && MINIMUM_REQUIRED_SIGNATURES=$num forge test --match-test testRegenerateBitField -vvv && forge test --match-test testRegenerateFiatShamirProofs
    popd

    # Generate final proof
    pnpm generateBeefyFinalProof
}

generate_beefy_gas_report() {
    num="$1"
    pushd ../../../contracts
    MINIMUM_REQUIRED_SIGNATURES=$num forge test --match-path test/BeefyClient.t.sol --gas-report
    popd
}


if [ -z "${from_benchmark:-}" ]; then
    generate_beefy_fixture 17
fi
