#!/usr/bin/env bash

set -eu
source ../web/packages/test/scripts/set-env.sh
rm -rf src/contracts
mkdir -p src/contracts

contracts_root="../contracts"
if [ "$snowbridge_v1" = "true" ]; then
        # Deploy legacy V1 contracts
        contracts_root="../../snowbridge-v1/contracts"
fi

# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGateway|IUpgradable|WETH9|MockGatewayV2|Token|HelloWorld|Greeter' \
    --bindings-path src/contracts \
    --root $contracts_root

# Install subxt
command -v subxt || cargo install subxt-cli \
    --git https://github.com/paritytech/subxt.git \
    --tag v0.43.0

eth_network="${ETH_NETWORK:-localhost}"
polkadot_network="${POLKADOT_NETWORK:-localhost}"

bridgehub_url="ws://localhost:11144"
assethub_url="ws://localhost:12144"
polkadot_url="ws://localhost:9944"
penpal_url="ws://localhost:13144"

if [ "$polkadot_network" == "westend" ]; then
  bridgehub_url="wss://westend-bridge-hub-rpc.polkadot.io"
  assethub_url="wss://westend-asset-hub-rpc.polkadot.io"
  polkadot_url="wss://westend-rpc.polkadot.io"
  penpal_url="wss://westend-penpal-rpc.polkadot.io"
fi


subxt codegen --url $bridgehub_url >src/parachains/bridgehub.rs \
  --derive ::subxt::ext::subxt_core::ext::codec::Encode \
  --derive ::subxt::ext::subxt_core::ext::codec::Decode
  subxt codegen --url $assethub_url > src/parachains/assethub.rs \
  --derive-for-type staging_xcm::v5::location::Location=Clone,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Encode,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Decode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Encode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Decode,recursive \
  --derive-for-type staging_xcm::v5::asset::AssetId=Clone,recursive \
  --derive-for-type staging_xcm::v5::asset::Assets=Clone,recursive
  subxt codegen --url $polkadot_url >src/parachains/relaychain.rs \
  --derive ::subxt::ext::subxt_core::ext::codec::Encode \
  --derive ::subxt::ext::subxt_core::ext::codec::Decode
  subxt codegen --url $penpal_url >src/parachains/penpal.rs \
  --derive-for-type staging_xcm::v5::location::Location=Clone,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Encode,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Decode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Encode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Decode,recursive \
  --derive-for-type staging_xcm::v5::asset::AssetId=Clone,recursive \
  --derive-for-type staging_xcm::v5::asset::Assets=Clone,recursive


