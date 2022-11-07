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
corepack prepare pnpm@latest --activate
```

Install all dependencies:

```
pnpm install
```

## Useful commands

Build all artifacts
```
npx turbo run build
```

Run all unit tests
```
npx turbo run test
```

Lint all the code
```
npx turbo run lint
```
