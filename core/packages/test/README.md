# Local Testnet

The E2E tests run against local deployments of the parachain, relayer, the ethereum execution layer (geth) and the ethereum consensus layer (lodestar).

## Requirements

* Ubuntu 20.04 or later. MacOS may work, but it's not currently a supported configuration.
* Development environment for Rust and Substrate. See parachain [requirements](../../../parachain/README.md#requirements).
* Development environment for the relay services. See setup [instructions](../../../relayer/README.md#Development).
* `jq` - https://stedolan.github.io/jq/download/
* geth - https://geth.ethereum.org/docs/install-and-build/installing-geth `go install github.com/ethereum/go-ethereum/cmd/geth@v1.10.23`
* g++ (required for lodestar) is available in the `build-essential` package: `apt install build-essential`
* sponge - Is available in the `moreutils` package.

  ```bash
  apt install moreutils
  ```

* direnv - https://direnv.net/

## Setup

### Install NPM dependencies

Make sure to install dependencies for all packages (contracts, api, test):

```bash
cd ../.. && pnpm install
```

### Polkadot

* Clone the polkadot repository somewhere on your machine
* Checkout tag `v0.9.30`.

Example:
```bash
git clone -n https://github.com/paritytech/polkadot.git
cd /path/to/polkadot
git fetch --tags
git checkout v0.9.30
cargo build --release
```

### Configure testnet

Create an `.envrc` file in which to hold environment config, using [.envrc-example](.envrc-example) as a template. Make sure to override the `POLKADOT_BIN` variable to the location of the polkadot binary built in the previous step.

Example:
```
POLKADOT_BIN=/home/sally/code/polkadot/target/release/polkadot
```

Once the `.envrc` has been created, let `direnv` load it automatically:

```bash
direnv allow
```

## Launch the testnet

Run the following script
```bash
scripts/start-services.sh
```

Wait until the "Testnet has been initialized" message

Go to polkadot-js and wait until the parachain has started producing blocks:
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A11144#/explorer

You can see the relay chain by connecting to https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer

Confirm the block number is > 2

## E2E tests

These tests are meant to closely replicate real-world behaviour. This means that they also replicate real-world delays and confirmation times. This can take up to 4 minutes per test and around 40 minutes for all tests.

### Run a specific test

To just run a specific test, the bridge needs to be bootstrapped first:

```bash
pnpm test:integration test/bootstrap.js
```

Now individual tests can be run, like the following:
```bash
pnpm test:integration --grep 'should transfer ETH from Substrate to Ethereum \(incentivized channel\)'
```

### Run all tests

Run the full suite of tests using the following command:

```bash
pnpm test:integration
```

The bootstrap tests will be called automatically as part of the full suite.

## Troubleshooting

The `start-services.sh` script writes the following logs:

- Parachain nodes: {alice,bob,11144,11155}.log
- Relay services: {beefy,parachain,beacon}-relay.log
- Geth (execution client): /tmp/snowbridge/geth.log
- Lodestar (beacon client): /tmp/snowbridge/lodestar.log

### Common issues

Sometimes during development tests will fail for transfers in the substrate->ethereum direction. If you see this, look in `parachain-relay.log` for the following error:
```
{"@timestamp":"2022-08-26T15:10:50.263740077+02:00","args":"[--api ws://localhost:11144 --block 0xe2e21a61b017699961b6d87c6aecbae18f2ce0c89bd87e0e8b0d808c26e2aad3]","level":"error","message":"Failed to query events.","name":"snowbridge-query-events","stdErr":"Error: Metadata(IncompatibleMetadata)\n","stdOut":""}
```

That means a dependency of the relayer has obsolete parachain metadata and needs to be refreshed. Please refer [here](../../../parachain/README.md#Chain_metadata) for steps to fix.

## Running E2E tests against Ropsten

To run the E2E tests on Ropsten you need to have separate accounts for the relayers, an account for deployment and one for running the E2E test stack. You will also require an [Infura](https://infura.io/) account and project.

Look at `.envrc-example` for the required variables. Add these variables to your `.envrc` and run `start-services.sh`.
