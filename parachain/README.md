# Polkadot Ethereum Parachain
![Check](https://github.com/Snowfork/polkadot-ethereum/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/polkadot-ethereum)

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

# Table of contents

* [Development](#local-development)
    * [Requirements](#requirements)
    * [Build](#build)
    * [Run](#run)
    * [Run with Docker](#run-with-docker)
    * [Configuration](#configuration)

## Development

Follow these steps to prepare your local environment for Substrate development.

### Requirements

The project is currently being developed and is working with the following version of Rust:
```
stable-x86_64-unknown-linux-gnu (default)
rustc 1.45.0 (5c1f21c3b 2020-07-13)
```

#### Simple Method

Install all the required dependencies with a single command (be patient, this can take up to 30
minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

#### Manual Method

Find manual setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Build

Once the development environment is set up, build the node template. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

### Run

Purge any existing dev chain state:

```bash
target/release/artemis-node purge-chain --dev
```

Start a dev chain:

```bash
target/release/artemis-node --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 target/release/artemis-node -lruntime=debug --dev
```

### Run with Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command (`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/artemis-node --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/artemis-node purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```

### Configuration

For a fully operational development chain, further configuration is required.

### Ethereum Contract Addresses

Application modules within the parachain are identified by hardcoded identifiers that match contract addresses for Bank applications on Ethereum. These identifiers are used for cross-chain message routing.

It is necessary to inject these addresses into the build environment so that our build [scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) can dynamically generate
Rust code for message dispatch.

Our parachain currently supports the following applications:

* ETH: A bridged ETH asset
* ERC20: Bridged ERC20 token assets

Correspondingly, the following environment variables need to be set:

```bash
export ETH_APP_ID=<CONTRACT_ADDR>
export ERC20_APP_ID=<CONTRACT_ADDR>
```

Example:

```bash
export ETH_APP_ID=0x0d27b0069241c03575669fed1badcbccdc0dd4d1
export ERC20_APP_ID=0x8fe1b1233f7032cef8cfc5eaaf411dffaa77a07c
```

Now rebuild the chain using the steps in [Build](#build).

Tip: Use [direnv](https://direnv.net/) to persist these variables in your development environment.

### Relayer Key

It is not required to change anything here for local development and testing.

The parachain depends on a external relayer service to forward messages to and from Ethereum. The relayer service is trusted by the parachain. Its identity should be injected into the [GenesisConfig](https://snowfork.github.io/artemis-rust-docs/pallet_verifier/struct.GenesisConfig.html#structfield.key) for the [Verifier](https://snowfork.github.io/artemis-rust-docs/pallet_verifier/index.html) pallet.

The node's baked-in chain spec uses `//Relay` as the relayer's account seed. For reference, see [chain_spec.rs](https://github.com/Snowfork/polkadot-ethereum/blob/main/parachain/node/src/chain_spec.rs#L50).
