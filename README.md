# Snowbridge

A trustless bridge between Polkadot and Ethereum.

## Development

To locally bootstrap our system for testing and development, or just to play around, please refer to our [tests](test/README.md) component.

## Components

### Ethereum

This component includes our Ethereum contracts, tests and truffle config.

See [ethereum/README.md](ethereum/README.md)

### Parachain

This component includes our substrate parachain, as well as our bridge-specific pallets.

See [parachain/README.md](parachain/README.md)

### Relayer

This component includes our Relayer daemon that will be run by relayers to watch and relay 2-way messages.

See [relayer/README.md](relayer/README.md)

### Tests

This component includes our end to end tests, that pull together all the above services and set them up easily through scripts for automated E2E tests.

See [test/README.md](test/README.md)

## Usage

To test out and use the bridge, please refer to the [Tests](#Tests) section above.

## Security

The security policy and procedures can be found in SECURITY.md.
