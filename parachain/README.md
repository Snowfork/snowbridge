# Snowbridge Parachain <!-- omit in toc -->
![Check](https://github.com/Snowfork/snowbridge/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/snowbridge)

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

- [Development](#development)
  - [Requirements](#requirements)
  - [Build](#build)
  - [Run](#run)
- [Configuration](#configuration)
  - [Ethereum Genesis Block](#ethereum-genesis-block)
  - [Ethereum Contract Addresses](#ethereum-contract-addresses)
- [API Documentation](#api-documentation)
- [Custom Types](#custom-types)

## Development

Follow these steps to prepare your local environment for Substrate development.

### Requirements

Refer to the instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

To add context to the above instructions, the parachain is known to compile with the following versions of Rust:

- stable: 1.51.0
- nightly: 1.53.0-nightly

### Build

Once the development environment is set up, build the parachain. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release --no-default-features --features with-local-runtime
```

### Run

Install `polkadot-launch`:

```bash
yarn global add polkadot-launch
cd -
```

Build polkadot:

```bash
git clone -n https://github.com/paritytech/polkadot.git /tmp/polkadot
cd /tmp/polkadot
git checkout release-v0.9.12
cargo build --release
cd -
```

Launch Polkadot and the parachain:

```bash
cd -
polkadot-launch config.json
```

It will take about 1-2 minutes for the parachain to start producing blocks.

The parachain will output logs to `200.log`.

## Configuration

For a fully operational chain, further configuration may be required.

### Ethereum Genesis Block

The parachain needs to be synced with the Ethereum chain before it can verify and dispatch Ethereum events. To bootstrap / sync the parachain quickly, it's advisable to set a newly finalized Ethereum block in the chain spec.

To get a newly finalized Ethereum block in a format compatible with Substrate's chain spec, use the `getblock` relayer command:

```bash
cd ../relayer
# Alternatively, use '--format rust' to get the Rust code
build/snowbridge-relay getblock --config /tmp/relay-config.toml --format json
```

Insert the output of the `getblock` command in the `initial_header` field in the `ethereum_light_client` section of the chain spec.

### Ethereum Contract Addresses

Each application module (ETH, ERC20) within the parachain must be configured with the contract address for its peer application on the Ethereum side. These addresses are included in Genesis storage via the chain spec.

For development and testing, it is not necessary to configure these. The builtin chain-spec already includes addresses that work out of the box with contracts deployed via `ganache-cli`.

To change the config to use your own addresses, follow these steps:

Generate a development chain-spec:

```bash
target/debug/snowbridge-node build-spec --dev > spec.json
```

Edit the generated spec file and replace the following addresses:

```json
      "ethApp": {
        "address": "0xfc97a6197dc90bef6bbefd672742ed75e9768553"
      },
      "erc20App": {
        "address": "0xeda338e4dc46038493b885327842fd3e301cab39"
      }
```

## API Documentation

See our [Rustdocs](https://polkaeth-rustdocs.netlify.app) for an overview of the crates, APIs, and types that make up our parachain.
