# Polkadot-Ethereum Bridge Contracts

This directory contains smart contracts utilized by the Polkadot-Ethereum Bridge.

## Set up

Create a `.env` file using the following template:

```bash
cp .env.example .env
```

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
