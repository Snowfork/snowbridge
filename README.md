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

## Development

We use the Nix package manager to provide a repeatable and maintainable developer environment.

After [installing](https://nixos.org/download.html) Nix, activate a developer shell here in the root of our repo, where `shell.nix` is located:

```
nix-shell
```

To ensure your code editor (such as VS code) can execute tools in the nix env, startup your editor within the interactive shell.

Example for VS Code:

```
nix-shell
code .
```

## Security

The security policy and procedures can be found in SECURITY.md.
