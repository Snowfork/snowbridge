# Snowbridge

A trustless bridge between Polkadot and Ethereum. For documentation, visit https://docs.snowbridge.network.

## Components

### Parachain

Polkadot parachain and our pallets. See [parachain/README.md](parachain/README.md).

### Relayer

Off-chain relayer services for relaying messages between Polkadot and Ethereum. See [relayer/README.md](relayer/README.md)

### Contracts

Ethereum contracts and unit tests. See [core/packages/contracts/README.md](core/packages/contracts/README.md)

### Integration Tests

This component includes our end to end tests, that pull together all the above services and set them up easily through scripts for automated E2E tests. See [core/packages/test/README.md](core/packages/test/README.md).

## Security

The security policy and procedures can be found in SECURITY.md.
