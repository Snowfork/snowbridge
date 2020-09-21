# Polkadot-Ethereum Bridge Contracts

This directory contains smart contracts utilized by the Polkadot-Ethereum Bridge.

## Contracts

- Bank: supports Ethereum and ERC20 deposits
- BankToken: enables ERC20 token testing

See Future Work section for core components in development

## Set up

Install dependencies

```bash
yarn install
```

Start truffle environment containing a local Ethereum network

```bash
truffle develop
```

Contract compilation

```bash
truffle compile
```

Migrate contracts to network

```bash
truffle migrate --reset --all
```

## Testing

Make sure the truffle environment is running

```bash
truffle develop
```

In another terminal, run tests

```bash
truffle test
```

Test gas expenditures

```bash
truffle test test/test_gas.js
```

## Future Work

- Polkadot Transaction Verifier
- Arbitrary State Parser
- Arbitrary State Submission
- Polkadot Signature Prover
