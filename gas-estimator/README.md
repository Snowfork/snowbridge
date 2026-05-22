# Snowbridge Gas Estimator

This tool calculates relaying profitability in the Ethereum -> Polkadot direction.

## Building

Runtime metadata (`.bin` files) is gitignored and must be generated before building. Install [subxt-cli](https://crates.io/crates/subxt-cli) then run:

```bash
# Polkadot (required for --features polkadot)
subxt metadata -f bytes --url wss://asset-hub-polkadot-rpc.n.dwellir.com > runtimes/asset-hub-polkadot/asset-hub-metadata.bin
subxt metadata -f bytes --url wss://bridge-hub-polkadot-rpc.n.dwellir.com > runtimes/bridge-hub-polkadot/bridge-hub-metadata.bin

# Paseo (for --features paseo)
subxt metadata -f bytes --url wss://sys.turboflakes.io/asset-hub-paseo > runtimes/asset-hub-paseo/asset-hub-metadata.bin
subxt metadata -f bytes --url wss://bridge-hub-paseo.dotters.network > runtimes/bridge-hub-paseo/bridge-hub-metadata.bin

# Westend (for --features westend)
subxt metadata -f bytes --url wss://westend-asset-hub-rpc.polkadot.io > runtimes/asset-hub-westend/asset-hub-metadata.bin
subxt metadata -f bytes --url wss://westend-bridge-hub-rpc.polkadot.io > runtimes/bridge-hub-westend/bridge-hub-metadata.bin
```

Then build with the desired feature, e.g.:

```bash
cargo build --release --features polkadot
```
