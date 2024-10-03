# Governance tools

Tools for generating governance proposals

## Example: Generate bridge activation preimage

```shell
cargo run --features polkadot --bin snowbridge-preimage -- \
  initialize \
  --exchange-rate-numerator 1 \
  --exchange-rate-denominator 400 \
  --multiplier-numerator 4 \
  --multiplier-denominator 3 \
  --fee-per-gas 80 \
  --local-reward 0.01 \
  --remote-reward 0.0001 \
  --checkpoint data/mainnet/initial-checkpoint.json \
  --gateway-address 0x1F98431c8aD98523631AE4a59f267346ea31F984 \
  --gateway-operating-mode normal
```

To target a different chain, replace `--features polkadot` with the applicable chain, e.g. `--features westend`.

The preimage can be tested using the generated `chopsticks-execute-upgrade.js` script.

NOTE: To test an upgrade that has not executed yet on the relevant environment, it can be tested using a local zombienet or chopsticks environment. Pass the `--bridge-hub-api` the `--asset-hub-api` params to override the default API endpoints.

# Update bindings

To update the runtime code binding, run the following commands:

```shell
subxt metadata --url ws://127.0.0.1:8000 -f bytes -o runtimes/polkadot/polkadot-metadata.bin
subxt metadata --url ws://127.0.0.1:8001 -f bytes -o runtimes/bridge-hub-polkadot/bridge-hub-metadata.bin
subxt metadata --url ws://127.0.0.1:8002 -f bytes -o runtimes/asset-hub-polkadot/asset-hub-metadata.bin
```

To update Westend/Paseo bindings, replace the chain name in the command, e.g. replace `runtimes/polkadot/polkadot-metadata.bin` 
with `runtimes/westend/polkadot-metadata.bin`.
