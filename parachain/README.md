# Polkadot Ethereum Parachain <!-- omit in toc -->
![Check](https://github.com/Snowfork/polkadot-ethereum/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/polkadot-ethereum)

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

- [Documentation](#documentation)
- [Development](#development)
  - [Requirements](#requirements)
  - [Dependencies](#dependencies)
  - [Build](#build)
  - [Configuration](#configuration)
    - [Ethereum Genesis Block](#ethereum-genesis-block)
    - [Ethereum Contract Addresses](#ethereum-contract-addresses)
  - [Run](#run)
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

### Build

Once the development environment is set up, build the parachain. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release --features "test-e2e"
```

### Configuration

For a fully operational chain, further configuration may be required.

#### Ethereum Genesis Block

The parachain needs to be synced with the Ethereum chain before it can verify and dispatch Ethereum events. To bootstrap / sync the
parachain quickly, it's advisable to set a newly finalized Ethereum block in the chain spec.

First, we need to generate the chain spec by running:
```bash
target/release/artemis build-spec --disable-default-bootnode > /tmp/spec.json
```

Next, we need to insert the appropriately transformed output from `eth_getBlockByNumber` RPC call from the Ganache chain into the genesis spec of our parachain. The easiest way to do this would be by running `curl` against our Ganache instance and using `transformEthHeader.js` script to transform it in appropriate format for the parachain spec:
```bash
curl http://localhost:8545 \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
    | node ../test/scripts/helpers/transformEthHeader.js
```

We can take the output from the above command and replace `genesis.runtime.verifierLightclient.initialHeader` of the previously generated spec in `/tmp/spec.json`.

A few more changes need to be made in order to ensure correct configuration:
1. `genesis.runtime.verifierLightclient.initialDifficulty` need to be set to `0x0`
2. `genesis.runtime.parachainInfo.parachainId` needs to be set to `200`
3. Finally `para_id` needs to be set to `200` also

You could also use `jq` if you have it installed to do the replacement:
```bash
jq --argjson header \
    "$(curl http://localhost:8545 \
    -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params": ["latest", false],"id":1}' \
    | node ../test/scripts/helpers/transformEthHeader.js)" \
    '.genesis.runtime.verifierLightclient.initialHeader = $header | .genesis.runtime.verifierLightclient.initialDifficulty = "0x0" | .genesis.runtime.parachainInfo.parachainId = 200 | .para_id = 200' \
    /tmp/spec.json > /tmp/parachain-spec.json
```

#### Ethereum Contract Addresses

Each application module (ETH, ERC20) within the parachain must be configured with the contract address for its peer application on the Ethereum side. These addresses are included in Genesis storage via the chain spec.

For development and testing, it is not necessary to configure these. The builtin chain-spec already includes addresses that work out of the box with contracts deployed via `ganache-cli`.

To change the config to use your own addresses, edit the previously generated spec file and replace the following addresses:

```json
"ethApp": {
  "address": "0xfc97a6197dc90bef6bbefd672742ed75e9768553"
},
"erc20App": {
  "address": "0xeda338e4dc46038493b885327842fd3e301cab39"
}
```

### Run


Install `polkadot-launch`:

```bash
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
yarn build
yarn global add file:$(pwd)
```

Build polkadot:

```bash
git clone https://github.com/paritytech/polkadot.git
cd polkadot
git checkout bf2d87a1
cargo build --release --features=real-overseer
```

Create a configuration for polkadot-launch by editing `parachain/config.json` and then copying into `polkadot-launch`.

You'll need to do the following substitutions:
1. Replace `relaychain.bin` with the location of your Polkadot checkout above
2. Replace `parachain.bin` with the path to your parachain binary
3. Replace `parachain.chain` with the path to your spec file

```bash
vim config.json
cp config.json /path/to/polkadot-launch
```

Launch polkadot and parachain:

```bash
polkadot-launch config.json
```

To view the parachain logs, open another terminal and view the `200.log` file. Note that it will take several minutes for the parachain to start producing blocks.

### Custom Types

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
