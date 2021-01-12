# Polkadot Ethereum Parachain <!-- omit in toc -->
![Check](https://github.com/Snowfork/polkadot-ethereum/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/polkadot-ethereum)

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

- [Documentation](#documentation)
- [Development](#development)
  - [Requirements](#requirements)
    - [Simple Method](#simple-method)
    - [Manual Method](#manual-method)
  - [Dependencies](#dependencies)
  - [Configuration](#configuration)
    - [Ethereum Contract Addresses](#ethereum-contract-addresses)
    - [Relayer Key](#relayer-key)
  - [Build](#build)
  - [Run](#run)
- [Interacting with the chain](#interacting-with-the-chain)
  - [Custom Types](#custom-types)

## Documentation

See our [Rustdocs](https://polkaeth-rustdocs.netlify.app) for an overview of the crates, APIs, and types that make up our parachain.

## Development

Follow these steps to prepare your local environment for Substrate development.

### Requirements

Find manual setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

The project currently follows the tip of Substrate master and so following nightly version of Rust is required:

```
nightly-2021-01-10-x86_64-unknown-linux-gnu (default)
```

This version is also specified in the `rust-toolchain` file and so it should be installed automatically when running cargo.

### Dependencies

Before building the parachain, ensure our smart contracts are deployed on your local truffle chain.

Follow the [Setup](../ethereum/README.md#set-up) guide to do this.

### Configuration

For a fully operational chain, further configuration may be required.

#### Ethereum Contract Addresses

Each application module (ETH, ERC20) within the parachain must be configured with the contract address for its peer application on the Ethereum side. These addresses are included in Genesis storage via the chain spec.

For development and testing, it is not necessary to configure these. The builtin chain-spec already includes addresses that work out of the box with contracts deployed via `ganache-cli`.

To change the config to use your own addresses, follow these steps:

Generate a development chain-spec:

```bash
target/debug/artemis-node build-spec --dev > spec.json
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

### Build

Once the development environment is set up, build the parachain. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

### Run


Install `polkadot-launch`:

```bash
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
yarn global add file:.
```

Build polkadot:

```bash
git clone https://github.com/paritytech/polkadot.git
cd polkadot
git checkout rococo-v1
cargo build --release --features=real-overseer
```

Create a configuration for polkadot-launch by editing `config.json`. You'll need to substitute `<POLKADOT_DIR>` with the location of your polkadot checkout above.

```bash
vim config.json
```

Launch polkadot and parachain:

```bash
polkadot-launch config.json
```

To view the parachain logs, open another terminal and view the `200.log` file. Note that it will take several minutes for the parachain to start producing blocks.


## Interacting with the chain

You can interact with a development chain using our [webapp](https://polkaeth-substrate.netlify.app). Its an instance of the Polkadot-JS webapp with the necessary configuration to interact with our development chain.

### Custom Types

For interacting with our chain using the Polkadot-JS API, you'll need to supply these custom types:

```json
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "AppId": "[u8; 20]",
  "Message": {
    "payload": "Vec<u8>",
    "verification": "VerificationInput"
  },
  "VerificationInput": {
    "_enum": {
      "Basic": "VerificationBasic",
      "None": null
    }
  },
  "VerificationBasic": {
    "blockNumber": "u64",
    "eventIndex": "u32"
  },
  "EthereumHeader": {
    "parentHash": "H256",
    "timestamp": "u64",
    "number": "u64",
    "author": "H160",
    "transactionsRoot": "H256",
    "ommersHash": "H256",
    "extraData": "Vec<u8>",
    "stateRoot": "H256",
    "receiptsRoot": "H256",
    "logBloom": "Bloom",
    "gasUsed": "U256",
    "gasLimit": "U256",
    "difficulty": "U256",
    "seal": "Vec<Vec<u8>>"
  },
  "Bloom": {
    "_": "[u8; 256]"
  },
  "AssetId": {
    "_enum": {
      "ETH": null,
      "Token": "H160"
    }
  }
}
```
