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
git clone https://github.com/paritytech/polkadot-launch.git /tmp/polkadot-launch
cd /tmp/polkadot-launch
yarn install
yarn build
yarn global add file:$(pwd)
```

Build polkadot:

```bash
git clone -n https://github.com/paritytech/polkadot.git /tmp/polkadot
cd /tmp/polkadot
git checkout bf2d87a1
cargo build --release --features=real-overseer
```

Start all services (parachain, relayer, ganache, etc):

```bash
scripts/start-services.sh
```

## Run Tests

### Integration Tests

NOTE: Integration tests are currently being redeveloped due to major architectural changes introduced recently in [#237](https://github.com/Snowfork/polkadot-ethereum/pull/237). So they are disabled for now.

```bash
yarn test
```

### Manual Tests

Make sure to setup the E2E stack as described above.

For interacting with Substrate, open [https://polkadot.js.org/apps/](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2Flocalhost%3A11144&types=eNplkm9PwjAQxr%2BK6StN9kKmLAQNCSjJSMAYBd8YYkp7bA1du%2FSPQcm%2Bu1e2BZiv2nv63K%2FX6x3ImHMD1pIhWXjpRBtGZK71zpfv2hsG%2F0%2BfcqoUyBknwwP5AuWLsJlQKxgZKi9lRGaKgXLiW%2FwCuoJWVRFZYD7NkHggnDqK5A9gj34wQmhptN6i8npc0Vxv0LqRmu1SanM8TeN%2Bgma3nykOexT8XYxxQ7sOuAY5iq7Oo5uAnLocDPgiBcrBBHZJDdbZgYsCrKNFGfDJPSr4wk3wNyH1LtchTHvJbUgwVFnKnNDKvmntTihdFGDsJR726H%2FuPh8vdHCZbICBKF0HKXU2kVpjy0m9RiSjdmVDn8mq9qAwF4UIWY3CxXYrGH7jz0mzQGVTQ9ulpklY77H7dZXhs7IXzSEMymfa78UPV%2FH67MtCdtqL8R2Y3hSHgxHcfoDefrIOJ2NrwXWGZrpM25FZ6h2otqlVVf0BYfrQ0w%3D%3D) in your browser (Make sure to use this link specifically).

#### Locking up ETH to mint PolkaETH on Substrate

Send 10 Ether to `//Alice` on Substrate:

```bash
cd ../ethereum
truffle exec scripts/sendEth.js 10
  0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d \
  --network e2e_test
```

It should take around 10-20 seconds for Substrate to receive and process the message.

In the substrate webapp (linked above), you should see an `Eth.Minted` event in the Network > Explorer view.

#### Burning PolkaETH to unlock ETH on Ethereum

To see the PolkaETH  balance for `//Alice`:

1. Navigate to Developer > Chain state > Storage
2. Select the `assets` module in the drop-down.
3. Select `ETH` as the AssetId
4. Select `Alice` as the AccountId
5. Click the `+` button

![Viewing the account balance for Alice](docs/query-balance.jpeg)

To burn PolkaETH and unlock ETH on Ethereum:

1. Navigate to Developer > Extrinsics
2. Select `Alice` as the AccountId
3. Select `eth` as the module, and `burn` as the extrinsic
4. Select `Basic` for the ChannelId
5. Input `0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD` for the recipient
6. Input `10000000000000000000` for the amount

![Viewing the account balance for Alice](docs/burn-polkaeth.jpeg)

It should take around 20 seconds for Ethereum to receive and process the message.

To verify that `0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD` received 10 Ether:

```bash
cd ../ethereum
truffle exec scripts/getEthBalance.js 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD --network e2e_test
```

## Debugging

The `start-services.sh` script writes the following logs:

* parachain.log
* relayer.log
* ganache.log
