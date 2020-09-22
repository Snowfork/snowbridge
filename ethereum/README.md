# Polkadot-Ethereum Bridge Contracts

This directory contains smart contracts utilized by the Polkadot-Ethereum Bridge.

- Bridge.sol: application registry and routing of messages from Substrate
- Application.sol: an abstract contract that must be implemented by any Bridge application
    - ETHApp.sol: application for cross-chain ETH transfers between Ethereum and Substrate
    - ERC20App.sol: application for cross-chain ERC20 transfers between Ethereum and Substrate
- Decoder.sol: a library for decoding SCALE encoded data
- Verifier.sol: verifies tx origin and signatures
- Scale.sol: implements decoding of SCALE encoded compact uints; not currently used, will support future work on generalized data relay

## Set up

After starting the blockchain and deploying the contracts, the Bridge's Ethereum component is ready for usage. To generate configuration information required for configuring the Parachain and Relayer components, run the set up scripts described in scripts section below.

```bash
# Install dependencies
yarn install
```

Start truffle environment with a local Ethereum network and deploy the contracts

```bash
truffle develop
truffle migrate --all
```

## Testing

Make sure the truffle environment is running, then run tests

```bash
# Start testing environment if it's not already running
truffle develop

# Test application gas expenditures
truffle test test/test_gas.js

# Run all tests
truffle test
```

## Scripts

To run the scripts, create an `.env` file using the example template.

```bash
cp scripts/.env.example scripts/.env
```

The project includes several scripts for Bridge interaction:

`getTx.js` gets information about a transaction

``` bash
truffle exec scripts/getTx.js [tx-hash]
```

`getEthBalance.js` gets an address' ETH balance

``` bash
truffle exec scripts/getEthBalance.js [ethereum-address]
```

`getERC20balance.js` gets an address' ERC20 token balance

``` bash
truffle exec scripts/getERC20Balance.js [ethereum-address] [token-contract-address]
```

`sendEth.js` deposits ETH into the ETHApp, initiating a cross-chain transfer to Substrate

``` bash
truffle exec scripts/sendEth.js [amount] [polkadot-recipient]
```

`sendERC20.js` deposits ERC20 tokens into the ERC20App, initiating a cross-chain transfer to Substrate

``` bash
truffle exec scripts/sendERC20.js [amount] [token-contract-address] [polkadot-recipient]
```

Several additional scripts support the set up of other components:

`dumpABI.js` outputs the ABI of the Bridge, ETHApp, and ERC20App contracts

``` bash
truffle exec scripts/dumpABI.js
```

`dumpParachainConfig.js`  outputs information required by the Parachain component

``` bash
truffle exec scripts/dumpParachainConfig.js
```

`dumpRelayerConfig.js`  outputs information required by the Relayer component

``` bash
truffle exec scripts/dumpRelayerConfig.js
```
