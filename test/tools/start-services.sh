#!/bin/bash

mkdir build
touch build/parachain.env

# Start Ganache
docker-compose up -d -- ganache
tools/wait-for-it.sh localhost:8545 -- echo "Ganache is up"

pushd ../ethereum

# Deploy contracts
truffle deploy --network ganache

# Generate configuration for relayer, parachain, and tests
truffle exec scripts/dumpRelayerDockerConfig.js --network ganache
truffle exec scripts/dumpParachainDockerConfig.js --network ganache
truffle exec scripts/dumpAddresses.js --network ganache
popd

# Start Parachain
docker-compose up -d -- parachain
tools/wait-for-it.sh localhost:9944 -- echo "Parachain is up"

# Start Relayer
docker-compose up -d -- relayer

