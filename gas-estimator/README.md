# Snowbridge Gas Estimator

This tool calculates relaying profitability in the Ethereum -> Polkadot direction.

After Asset Hub or Bridge Hub runtime upgrades, rebuild the estimator (and the
relayer Docker image) with freshly fetched metadata. Stale subxt codegen fails
at runtime with `Metadata(IncompatibleCodegen)` on dry-run APIs. The relayer
Dockerfile re-fetches metadata on every release build via
`GAS_ESTIMATOR_METADATA_CACHEBUST`, and the scheduled `check-runtime-upgrade`
workflow compares live spec versions against the labels on the published image,
auto-dispatching a `release-relayer` rebuild when they drift. Redeploying the
running relayer with the rebuilt image is still a manual step — until then the
relayer logs `ALERT: gas estimator metadata incompatible with live runtime`.

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
