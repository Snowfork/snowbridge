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

- stable: 1.58
- nightly: 1.60.0-nightly

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
yarn global add polkadot-launch@1.9.0
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

## Configuration

Note: This section is not necessary for local development, as there are scripts to auto-configure the parachain.

For a fully operational chain, further configuration of the initial chain spec is required. The specific configuration will depend heavily on your environment, so this guide will remain high-level.

Build an initial spec:
```bash
target/debug/snowbridge build-spec --disable-default-bootnode > spec.json
```

Now edit the spec and configure the following:
1. Recently finalized ethereum header and difficulty for the ethereum light client
2. Contract addresses for the Ether, Erc20, and Dot apps.
3. Authorized principal for the basic channel
4. Fee and reward parameters for the incentivized channel

For an example configuration, consult the [setup script](https://github.com/Snowfork/snowbridge/blob/main/test/scripts/start-services.sh) for our local development stack. Specifically the `start_polkadot_launch` bash function.
