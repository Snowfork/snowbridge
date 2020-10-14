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

## Documentation

See our [Rustdocs](https://sad-curie-a48c3f.netlify.app/) for an overview of the crates, APIs, and types that make up our parachain.

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

For a fully operational development chain, further configuration is required.

#### Ethereum Contract Addresses

Application modules within the parachain are identified by hardcoded identifiers that match contract addresses for Bank applications on Ethereum. These identifiers are used for cross-chain message routing.

It is necessary to inject these addresses into the build environment so that our build [scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) can dynamically generate
Rust code for message dispatch.

Autmatically:

```bash
eval $(scripts/make-build-config.sh)

# verify that the environment variables are set
echo $ETH_APP_ID
echo $ERC20_APP_ID
```

Or manually (replace example addresses with your own):

```bash
export ETH_APP_ID=0x0d27b0069241c03575669fed1badcbccdc0dd4d1
export ERC20_APP_ID=0x8fe1b1233f7032cef8cfc5eaaf411dffaa77a07c
```

Tip: Use [direnv](https://direnv.net/) to persist these variables in your development environment.

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
target/release/artemis-node --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 target/release/artemis-node -lruntime=debug --dev
```

## Interacting with the chain

You can interact with a development chain using our [webapp](https://xenodochial-goldstine-1ba19f.netlify.app). Its an instance of the Polkadot-JS webapp with the necessary configuration to interact with our development chain.

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
  "TokenId": "H160",
  "BridgedAssetId": "H160",
  "AssetAccountData": {
    "free": "U256"
  }
}
```
