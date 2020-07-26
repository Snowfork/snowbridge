# Bridgerelayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packets to the correlated bridge component.

Note: the bridgerelayer is currently in a boilerplate/architectural design state, it's not functional yet.

## Setup

```bash
export GO111MODULE=on
export GOPROXY=direct
export GOSUMDB=off

make install
```

For testing, start a local Ethereum network and deploy the Bank contract by following the set up instructions [here](../ethereum/README.md).

## Usage

```bash
# Check that the binary was successfully installed
bridgerelayer -h

# Start the relayer
bridgerelayer init wss://rpc.polkadot.io ws://localhost:7545/
```

You should see a message similar to
```bash
INFO[0000] Connected to Ethereum chain ID 5777          
INFO[0000] Subscribed to app 0xC4cE93a5699c68241fc2fB503Fb0f21724A624BB 
```

You can send a `sendEth` transaction to the Bank contract with default values via the sendEth script located in polkadot-ethereum/ethereum/scripts/sendEth.js

```bash
# Send the transaction
truffle exec sendEth.js

# You should see the transaction in the bridgerelayer
INFO[0007] Witnessed tx 0x22c26a2d423bcc9622daba9410f5bdee1d047ec2e8be5c112a01b64224dbea5e on app 0xC4cE93a5699c68241fc2fB503Fb0f21724A624BB 
```

Currently, the relayer logs the packet instead of sending it directly to the bridge. It should look similar to
```bash
INFO[0007] Send packet:
{[196 206 147 165 105 156 104 36 31 194 251 80 63 176 242 23 36 166 36 187 0 0 0 0 0 0 0 0 0 0 0 0] {[249 1 250 148 196 206 147 165 105 156 104 36 31 194 251 80 63 176 242 23 36 166 36 187 225 160 38 100 19 190 87 0 206 141 213 172 107 154 125 251 171 233 155 62 69 202 233 166 138 194 117 120 88 113 11 64 26 56 185 1 192 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 96 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 192 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 46 99 111 109 112 101 108 116 101 108 121 32 100 105 102 102 101 114 101 110 116 32 105 100 101 110 116 105 102 105 99 97 116 105 111 110 32 117 110 105 113 117 101 32 110 111 119 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 7 115 101 110 100 69 84 72 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 136 17 95 110 32 4 215 180 204 214 185 213 171 52 227 9 9 224 246 18 205 116 104 105 115 32 105 115 32 110 111 116 32 97 110 121 116 104 105 110 103 32 108 105 107 101 32 98 101 102 111 114 101 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 64 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0] [101 109 112 116 121]}}
```


## Previous work

Thanks to Chainsafe for their work on [ChainBridge](https://github.com/ChainSafe/ChainBridge), a event-based bridge relayer that this project is based on.

