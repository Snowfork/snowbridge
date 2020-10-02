# Polkadot Ethereum Bridge

Components for a Polkadot Ethereum Bridge

## Components

### Ethereum

This folder includes our Ethereum contracts, tests and truffle config.

See [ethereum/README.md](ethereum/README.md)

### Parachain

This folder includes our substrate parachain, as well as our bridge-specific pallets.

See [parachain/README.md](parachain/README.md)

### Relayer

This folder includes our Relayer daemon that will be run by relayers to watch and relay 2-way messages.

See [relayer/README.md](relayer/README.md)

### Tests

This folder includes our end to end tests, that pull together all the above services and set them up easily through scripts for automated E2E tests.

See [test/README.md](test/README.md)

## Usage

To test out and use the bridge, see each of the above READMEs in order and run through their steps, or just look through the test guide if that's all you need. The full functionality can then also be demonstrated using our [fork](https://github.com/Snowfork/substrate-ui) of the Polkadot-JS web application. Extra demo steps described [here](https://github.com/Snowfork/substrate-ui/tree/stable-base/packages/app-polkadot-ethereum-bridge).

## Security

The security policy and procedures can be found in SECURITY.md.
