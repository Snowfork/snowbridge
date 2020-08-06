
# Polkadot Ethereum Parachain

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

# Table of contents

* [Development](#local-development)
    * [Simple Method](#simple-method)
    * [Manual Method](#manual-method)
    * [Build](#build)
* [Run](#run)
    * [Single Node Development Chain](#single-node-development-chain)
    * [Multi-Node Local Testnet](#multi-node-local-testnet)
    * [Run in Docker](#run-in-docker)
* [Interacting with the chain](#interacting-with-the-chain)


## Local Development

Follow these steps to prepare your local environment for Substrate development :hammer_and_wrench:

### Requirements

The project is currently being developed and is working with the following version of Rust:
```
stable-x86_64-unknown-linux-gnu (default)
rustc 1.45.0 (5c1f21c3b 2020-07-13)
```

### Simple Method

You can install all the required dependencies with a single command (be patient, this can take up
to 30 minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Method

Manual steps for Linux-based systems can be found below; you can
[find more information at substrate.dev](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

### Build

Use this command to build the [Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution)
and [native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/artemis-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/artemis-node --dev
```

Detailed logs may be shown by running the node with the following environment variables set:
`RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

### Run in Docker

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

See https://polkadot.js.org/api/start/FAQ.html#i-cannot-send-transactions-from-my-node-template-based-chain for more details.
