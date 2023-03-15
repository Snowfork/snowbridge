# Local Testnet

The E2E tests run against local deployments of the parachain, relayer, the ethereum execution layer (geth) and the ethereum consensus layer (lodestar).

## Requirements

The E2E stack can be run on any system that supports the [Nix](https://nixos.org/explore.html) package manager. This
includes Linux, MacOS, and Windows (WSL2). See the [main README's Development section](../../../README.md#Development) for
setup instructions. Ensure that you are in a Nix development shell for the remaining instructions.

## Setup

### Configure testnet

All required environment variables have reasonable defaults so normally there is no need to configure them. If necessary, you can override them with an `.envrc` file, using [.envrc-example](.envrc-example) as a template.

Once the `.envrc` has been created, let `direnv` load it automatically:

```bash
direnv allow
```

## Launch the testnet

Run the following script:

```bash
scripts/start-services.sh
```

Wait until the "Testnet has been initialized" message.

Go to polkadot-js and wait until the parachain has started producing blocks:
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A11144#/explorer

You can see the relay chain here:
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer

Confirm the block number is > 2.

### Using a custom Cumulus build for BridgeHub and Statemine

By default `scripts/start-services.sh` will use `cargo` to install the `polkadot-parachain` binary from the [Snowbridge/cumulus](https://github.com/Snowfork/cumulus) forks `snowbridge` branch.

To use a custom version of cumulus:

1. Clone a cumulus repo. In the example below we clone the [Snowbridge/cumulus](https://github.com/Snowfork/cumulus) fork.

```console
git clone https://github.com/Snowfork/cumulus.git -b snowbridge
```

2. Build `polkadot-parachain` bin.

```console
cd cumulus
cargo build --release --bin polkadot-parachain
```

3.  Set `CUMULUS_BIN` in `core/packages/test/.envrc` and `direnv allow`.

```sh
export CUMULUS_BIN=/path/to/cumulus/target/release/polkadot-parachain
```

## E2E tests

These tests are meant to closely replicate real-world behaviour. This means that they also replicate real-world delays and confirmation times. This can take up to 4 minutes per test and around 40 minutes for all tests.

### Run a specific test

To just run a specific test, the bridge needs to be bootstrapped first:

```bash
pnpm test:bootstrap
```

Now individual tests can be run, like the following:

```bash
pnpm test:integration --grep 'should transfer ETH from Ethereum to Substrate \(basic channel\)'
```

### Run all tests

Run the full suite of tests using the following command:

```bash
pnpm test:integration
```

The bootstrap tests will be called automatically as part of the full suite.

## Troubleshooting

The `start-services.sh` script writes the following logs:

- BridgeHub nodes: {alice,bob,11144,11155}.log
- Statemine nodes: {alice,bob,11144,11155}.log
- Relay services: {beefy,parachain,beacon}-relay.log
- Geth (execution client): /tmp/snowbridge/geth.log
- Lodestar (beacon client): /tmp/snowbridge/lodestar.log

### Common issues

Sometimes during development tests will fail for transfers in the substrate->ethereum direction. If you see this, look in `parachain-relay.log` for the following error:
```
{"@timestamp":"2022-08-26T15:10:50.263740077+02:00","args":"[--api ws://127.0.0.1:11144 --block 0xe2e21a61b017699961b6d87c6aecbae18f2ce0c89bd87e0e8b0d808c26e2aad3]","level":"error","message":"Failed to query events.","name":"snowbridge-query-events","stdErr":"Error: Metadata(IncompatibleMetadata)\n","stdOut":""}
```

That means a dependency of the relayer has obsolete parachain metadata and needs to be refreshed. Please refer [here](../../../parachain/README.md#Chain_metadata) for steps to fix.

## Running E2E tests against Ropsten

To run the E2E tests on Ropsten you need to have separate accounts for the relayers, an account for deployment and one for running the E2E test stack. You will also require an [Infura](https://infura.io/) account and project.

Look at `.envrc-example` for the required variables. Add these variables to your `.envrc` and run `start-services.sh`.
