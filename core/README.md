# Core Typescript Packages

Packages:

- Solidity Contracts
- API client
- Integration tests

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

Install [foundry](https://book.getfoundry.sh/getting-started/installation):

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Install all dependencies:

```bash
pnpm install
```

## Useful commands

Build all artifacts:

```bash
pnpm build
```

Run all unit tests:

```bash
pnpm test
```

Lint all the code:

```bash
pnpm lint
```
