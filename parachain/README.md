# Snowbridge Parachain <!-- omit in toc -->
![Check](https://github.com/Snowfork/snowbridge/workflows/Check/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/Snowfork/polkadot-ethereum/badge.svg)](https://coveralls.io/github/Snowfork/snowbridge)

A Polkadot parachain for bridging arbitrary data from and to Ethereum.

- [Development](#development)
  - [Requirements](#requirements)
  - [Build](#build)
  - [Run](#run)
- [Configuration](#configuration)

## Development

Follow these steps to prepare your local environment for Substrate development.

### Requirements

Refer to the instructions at the
[Substrate Developer Hub](https://docs.substrate.io/main-docs/install/).

To add context to the above instructions, the parachain is known to compile with the following versions of Rust:

- stable: 1.62.1
- nightly: 1.64.0-nightly

### Build

Once the development environment is set up, build the parachain. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

Several runtimes can be built:
* snowbase: Local development
* snowblink: Staging & Kusama parachain
* snowbridge: Polkadot parachain

To build with snowbase and snowblink runtimes (the default):

```bash
cargo build --release --features rococo-native
```

## Configuration

Note: This section is not necessary for local development, as there are scripts to auto-configure the parachain.

For a fully operational chain, further configuration of the initial chain spec is required. The specific configuration will depend heavily on your environment, so this guide will remain high-level.

Build an initial spec for the snowbase runtime:

```bash
target/release/snowbridge build-spec --chain snowbase --disable-default-bootnode > spec.json
```

Now edit the spec and configure the following:
1. Recently finalized ethereum header and difficulty for the ethereum light client
2. Contract addresses for the Ether, Erc20, and Dot apps.
3. Authorized principal for the basic channel
4. Fee and reward parameters for the incentivized channel

For an example configuration, consult the [setup script](https://github.com/Snowfork/snowbridge/blob/main/test/scripts/start-services.sh) for our local development stack. Specifically the `start_polkadot_launch` bash function.

## Tests

To run the parachain tests locally, use `cargo test --release`. For the full suite of tests, use `cargo test --release --features runtime-benchmarks`.

### Updating test data for inbound channel unit tests

To regenerate the test data, use a test with multiple `submit` calls in `ethereum/test/test_{basic,incentivized}_outbound_channel.js`.
For each encoded log you want to create, find a transaction object `tx` returned from a `submit` call, then run this:

```javascript
const rlp = require("rlp");
const rawLog = tx.receipt.rawLogs[0];
const encodedLog = rlp.encode([rawLog.address, rawLog.topics, rawLog.data]).toString("hex");
console.log(`encodedLog: ${encodedLog}`);
```

To decode the event from a `rawLog` to inspect the event data:

```javascript
const iface = new ethers.utils.Interface(BasicOutboundChannel.abi);
const decodedEventLog = iface.decodeEventLog(
  'Message(address,address,uint64,bytes)',
  rawLog.data,
  rawLog.topics,
);
console.log(`decoded rawLog.data: ${JSON.stringify(decodedEventLog)}`);
```

Set the contract object and event signature based on the log you want to decode.

## Chain metadata

There is an internal tool `snowbridge-query-events` which is used to read specific events from the parachain. It is a used by our offchain message relayers.

This tool must be kept up to date with the latest chain metadata. This is the process for keeping that up to date:

Install subxt client:

```bash
cargo install subxt-cli
```

Update metadata by fetching it from parachain node (in this case a node in the E2E stack):
```
subxt metadata --url http://localhost:8081 -f bytes > tools/query-events/metadata.scale
```

If you want to update the tool for an already running E2E stack:

```
cargo build --release --manifest-path tools/query-events/Cargo.toml
cp target/release/snowbridge-query-events /tmp/snowbridge/bin/
```
