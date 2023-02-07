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

After [installing nix](https://nixos.org/download.html) Nix, enable [flakes](https://nixos.wiki/wiki/Flakes):

```sh
mkdir -p ~/.config/nix
echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
```

Then activate a developer shell in the root of our repo, where [`flake.nix`](./flake.nix) is located:

```
nix develop --command $SHELL
```

To ensure your code editor (such as VS Code) can execute tools in the nix shell, startup your editor within the interactive shell.

Example for VS Code:

```
nix develop --command $SHELL
code .
```

To automatically enter this nix shell whenever you open the project, install [`direnv`](https://direnv.net/docs/installation.html) and run `direnv allow` in the
project root.

## Security

The security policy and procedures can be found in SECURITY.md.
