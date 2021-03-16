# Polkadot Ethereum Parachain <!-- omit in toc -->
![Check](https://github.com/Snowfork/polkadot-ethereum/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/polkadot-ethereum)

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

- stable: 1.50.0
- nightly: 1.52.0-nightly

### Build

Once the development environment is set up, build the parachain. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
--no-default-features --features with-local-runtime
```

### Run

Install `polkadot-launch`:

```bash
yarn global add polkadot-launch
```

Build Polkadot:

```bash
git clone -n https://github.com/paritytech/polkadot.git /tmp/polkadot
cd /tmp/polkadot
git checkout 8daf97
cargo build --release --features=real-overseer
```

Launch Polkadot and the parachain:

```bash
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
build/artemis-relay getblock --config /tmp/relay-config.toml --format json
```

Insert the output of the `getblock` command in the `initial_header` field in the `verifier_lightclient` section of the chain spec.

### Ethereum Contract Addresses

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

## API Documentation

See our [Rustdocs](https://polkaeth-rustdocs.netlify.app) for an overview of the crates, APIs, and types that make up our parachain.

## Custom Types

For interacting with our chain using the Polkadot-JS API, you'll need to supply these custom types:

```json
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "ChannelId": {
    "_enum": {
      "Basic": null,
      "Incentivized": null
    }
  },
  "MessageNonce": "u64",
  "MessageId": {
    "channelId": "ChannelId",
    "nonce": "u64"
  },
  "Message": {
    "data": "Vec<u8>",
    "proof": "Proof"
  },
  "Proof": {
    "blockHash": "H256",
    "txIndex": "u32",
    "data": "(Vec<Vec<u8>>, Vec<Vec<u8>>)"
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
  "EthashProofData": {
    "dagNodes": "[H512; 2]",
    "proof": "Vec<H128>"
  },
  "Bloom": {
    "_": "[u8; 256]"
  },
  "PruningRange": {
    "oldestUnprunedBlock": "u64",
    "oldestBlockToKeep": "u64"
  },
  "AssetId": {
    "_enum": {
      "ETH": null,
      "Token": "H160"
    }
  },
  "InboundChannelData": {
    "nonce": "u64"
  },
  "OutboundChannelData": {
    "nonce": "u64"
  }
}
```
