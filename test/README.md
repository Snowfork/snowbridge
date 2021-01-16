# E2E tests

The E2E tests run against local deployments of the parachain, relayer and ganache.

## Requirements

1. Development environment for Rust and Substrate. See parachain [requirements](../parachain/README.md#requirements).
2. Development environment for Ethereum smart contracts.

   ```bash
   yarn global add truffle
   (cd ../ethereum && yarn install)
    ```

3. `timeout` - native package on Ubuntu, on macOS try ```brew install coreutils```

## Setup

Download dependencies:

```bash
yarn install
```

Install `polkadot-launch`:

```bash
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
yarn global add file:$(pwd)
```

Build polkadot:

```bash
git clone https://github.com/paritytech/polkadot.git
cd polkadot
git checkout rococo-v1
cargo build --release --features=real-overseer
```

Update `../parachain/config.json`. You'll need to substitute `<POLKADOT_DIR>` with the location of your polkadot checkout above.


Start all services (parachain, relayer, ganache, etc):

```bash
scripts/start-services.sh
```

## Run Tests

```bash
yarn test
```

## Debugging

The `start-services.sh` script writes the following logs:

* parachain.log
* relayer.log
* ganache.log
