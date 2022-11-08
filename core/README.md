# Core Typescript Packages

Packages:
* Solidity Contracts
* API client
* Integration tests

## Getting Started

Install [nvm](https://github.com/nvm-sh/) and activate nodejs:

```
nvm use
```

Install [pnpm](https://pnpm.io/):

```
corepack enable
corepack prepare pnpm@7.14.2 --activate
```

Install all dependencies:

```
pnpm install
```

## Useful commands

Build all artifacts
```
pnpm run build
```

Run all unit tests
```
pnpm run test
```

Lint all the code
```
pnpm run lint
```
