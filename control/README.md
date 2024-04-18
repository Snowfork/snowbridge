# Governance tools

Tools for generating governance proposals

## Example: Generate bridge activation preimage

```shell
snowbridge-preimage
  --bridge-hub-api ws://localhost:8001 \
  --asset-hub-api ws://localhost:8000 \
  initialize \
  --exchange-rate-numerator 1 \
  --exchange-rate-denominator 400 \
  --multiplier-numerator 4 \
  --multiplier-denominator 3 \
  --fee-per-gas 80 \
  --local-reward 0.01 \
  --remote-reward 0.0001 \
  --checkpoint initial-checkpoint.json \
  --gateway-address 0x1F98431c8aD98523631AE4a59f267346ea31F984 \
  --gateway-operating-mode normal
```

The preimage can be tested using the generated `chopsticks-execute-upgrade.js` script

NOTE: Since the 1.2.0 upgrade has not executed yet on mainnet Polkadot, I tested the tool using a local zombienet or chopsticks environment. Pass the `--bridge-hub-api` the `--asset-hub-api` params to override the default API endpoints.

## Update runtime bindings

To generate runtime bindings that include the 1.2.0 runtime release, we need to start a local `polkadot-local` network using zombienet.

Build polkadot executables:

```shell
cd $WORKSPACE/polkadot-sdk
cargo build --release
cp target/release/{polkadot,polkadot-prepare-worker,polkadot-execute-worker,polkadot-parachain} $WORKDIR/
```

Build the `chain-spec-generator` for production runtimes:

```shell
cd $WORKSPACE/runtimes
cargo build -p chain-spec-generator --profile production
cp target/production/chain-spec-generator $WORKDIR/
```

Create initial chainspecs:

```shell
chain-spec-generator polkadot-local > polkadot-local.json
chain-spec-generator asset-hub-polkadot-local > asset-hub-polkadot-local.json
chain-spec-generator bridge-hub-polkadot-local > bridge-hub-polkadot-local.json
```

Launch zombienet:

```shell
zombienet spawn launch-config.toml
```

Update bindings:

```shell
subxt metadata --url ws://127.0.0.1:8000 -f bytes -o runtimes/polkadot/polkadot-metadata.bin
subxt metadata --url ws://127.0.0.1:8001 -f bytes -o runtimes/bridge-hub-polkadot/bridge-hub-metadata.bin
subxt metadata --url ws://127.0.0.1:8002 -f bytes -o runtimes/asset-hub-polkadot/asset-hub-metadata.bin
```
