# Polkadot Ethereum Bridge
Components for a Polkadot Ethereum Bridge

## Components

### Parachain
This folder includes our substrate parachain, as well as our bridge-specific pallets.

See [parachain/README.md](parachain/README.md)

### Ethereum
This folder includes our Ethereum contracts, tests and truffle config.

See [ethereum/README.md](ethereum/README.md)

### Relayer
This folder includes our Relayer daemon that will be run by relayers to watch and relay 2-way messages.

See [relayer/README.md](relayer/README.md)

### Usage
To test out and use the bridge, see each of the above READMEs in order and run through their steps. The full functionality can then also be demo'd using our substrate-ui fork, with extra demo steps described [here](https://github.com/Snowfork/substrate-ui/tree/stable-base/packages/app-polkadot-ethereum-bridge)
