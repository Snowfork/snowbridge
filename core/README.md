# Core Typescript Packages

Packages:
* Solidity Contracts
* API client
* Integration tests

## Getting Started

Install [nvm](https://github.com/nvm-sh/) and activate nodejs:

```bash
nvm use
```

Install [pnpm](https://pnpm.io/):

```bash
corepack enable
corepack prepare pnpm@7.14.2 --activate
```

Install all dependencies:

```bash
pnpm install
```

## Useful commands

Build all artifacts:

```bash
pnpm run build
```

Run all unit tests:

```bash
pnpm run test
```

Lint all the code:

```bash
pnpm run lint
```
