# Bridgerelayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packets to the correlated bridge component.

Note: the bridgerelayer is currently in a boilerplate/architectural design state, it's not functional yet.

## Setup

```bash
make install
```

## Usage

```bash
# Check that the binary was successfully installed
bridgerelayer -h

# Start the relayer
bridgerelayer init wss://rpc.polkadot.io wss://mainnet.infura.io/ws/v3/{INFURA_PROJECT_ID}
```

## Previous work

Thanks to Chainsafe for their work on [ChainBridge](https://github.com/ChainSafe/ChainBridge), a event-based bridge relayer that this project is based on.

