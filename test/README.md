# E2E tests

The E2E tests run against dockerized services.

## Requirements

1. Development environment for Rust and Substrate. See parachain [requirements](../parachain/README.md#requirements).
2. Development environment for Ethereum smart contracts.

   ```bash
   yarn global add truffle
   (cd ../ethereum && yarn install)
    ```

3. Docker and docker-compose

## Setup

Currently, setting up the dockerized environment is partially automated. Full automation requires further work due to some intricacies in our configuration and build process.

Download dependencies:

```bash
yarn install
```

Run the following commands one by one:

```bash
mkdir build
mkdir build/parachain-state
touch build/parachain.env

# Start ganache service
docker-compose up -d -- ganache

# Ensure ganache starts up successfully
docker-compose logs ganache | tail -n 20

# change to Ethereum contracts dir
pushd ../ethereum

# Deploy contracts
truffle deploy --network ganache

# Generate configuration for relayer, parachain, and tests
truffle exec scripts/dumpRelayerDockerConfig.js --network ganache
truffle exec scripts/dumpParachainDockerConfig.js --network ganache
truffle exec scripts/dumpAddresses.js --network ganache

# Change back to previous directory
popd

# Start Parachain
docker-compose up -d -- parachain

# Wait until parachain compiles and starts up successfully
docker-compose logs parachain | tail -n 20

# Start Relayer
docker-compose up -d -- relayer

# Wait until relayer starts up successfully
docker-compose logs relayer | tail -n 20
```

## Run Tests

```bash
yarn test
```
