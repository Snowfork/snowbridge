# Local Testnet

The E2E tests run against local deployments of the parachain, relayer and ethereum (geth).

## Requirements

* Ubuntu 20.04 or later. MacOs may work, but its not currently a supported configuration. 
* Development environment for Rust and Substrate. See parachain [requirements](../parachain/README.md#requirements).
* Node 14 LTS. Can install using [nvm](https://github.com/nvm-sh/nvm#installing-and-updating):

  ```bash
  nvm install 14.17.4
  nvm use 14.17.4
  ```

* Development environment for Ethereum smart contracts.

  ```bash
  (cd ../ethereum && yarn install)
  ```

* Development environment for the relay services. See setup [instructions](../relayer/README.md#development).
* `jq` - https://stedolan.github.io/jq/download/
* geth - https://geth.ethereum.org/docs/install-and-build/installing-geth
* sponge - Is available in the `moreutils` package.

  ```bash
  apt install moreutils
  ```

* polkadot-launch

  ```bash
  yarn global add polkadot-launch@1.9.0
  ```

## Setup

### Install NPM dependencies

```bash
yarn install
```

### Polkadot

* Clone the polkadot repository somewhere on your machine
* Checkout commit `release-v0.9.12`.

Example:
```bash
git clone -n https://github.com/paritytech/polkadot.git
cd /path/to/polkadot
git checkout release-v0.9.12
cargo build --release
```

### Configure testnet

Create an `.env` file, and set the `POLKADOT_BIN` variable to the location of the polkadot binary built in the previous step.

Example:
```
POLKADOT_BIN=/path/to/polkadot/target/release/polkadot
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

### Troubleshooting

The `start-services.sh` script writes the following logs:

- Parachain nodes: /tmp/snowbridge/{alice,bob,11144,11155}.log
- Relay services: /tmp/snowbridge/{beefy,parachain,ethereum}-relay.log
- Geth: /tmp/snowbridge/geth.log

## E2E tests

Run the tests using the following command:

```bash
yarn test
```

These tests are meant to closely replicate real-world behaviour. This means that they also replicate real-world delays and confirmation times. This can take up to 4 minutes per test and ~20minutes for all tests.

### Testing against a malicious contract
We also have a test environment that tests against a malicious contract that attempts to consume infinite gas. To setup this environment, run the start-services script with the malicious flag:

```bash
TEST_MALICIOUS_APP=1 scripts/start-services.sh
```

This will deploy and run everything as usual, but replace the dot app with a malicious one. Once everything is ready to go, run the tests for the malicious app:

```bash
yarn test ./test/malicious-dotapp.js
```

You should see the test pass, checking that message delivery works correctly and channel functionality is still secure without being affected by the malicious app.


## Generating/Updating new test fixtures

Test fixtures are taken by running the service in full e2e test. The relayer should log the fixture data you need (code is in [the relayer here](../relayer/workers/beefyrelayer/fixture-data-logger.go), though may require a bit of manual copy/pasting to get perfectly it in the right format.
