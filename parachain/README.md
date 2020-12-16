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

#### Relayer Key

_It is not required to change anything here for local development and testing._

The parachain depends on a external relayer service to forward messages to and from Ethereum. The relayer service is trusted by the parachain. Its identity should be injected into the [GenesisConfig](https://snowfork.github.io/artemis-rust-docs/pallet_verifier/struct.GenesisConfig.html#structfield.key) for the [Verifier](https://snowfork.github.io/artemis-rust-docs/pallet_verifier/index.html) pallet.

The node's baked-in chain spec uses `//Relay` as the relayer's account seed. For reference, see [chain_spec.rs](https://github.com/Snowfork/polkadot-ethereum/blob/main/parachain/node/src/chain_spec.rs#L50).

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
target/release/artemis-node --tmp --dev
```

Or, start a dev chain with a custom chain spec:

```bash
target/release/artemis-node --tmp --spec spec.json
```

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
