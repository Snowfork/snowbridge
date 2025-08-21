#!/usr/bin/env bash

set -eu
rm -rf src/contracts
mkdir -p src/contracts

contracts_root="../contracts"
if [ "$snowbridge_v1" = "true" ]; then
        # Deploy legacy V1 contracts
        contracts_root="../../snowbridge-v1/contracts"
fi

# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGateway|IUpgradable|WETH9|MockGatewayV2|Token|HelloWorld' \
    --bindings-path src/contracts \
    --root $contracts_root

# Install subxt
command -v subxt || cargo install subxt-cli \
    --git https://github.com/paritytech/subxt.git \
    --tag v0.43.0

eth_network="${ETH_NETWORK:-localhost}"
polkadot_network="${POLKADOT_NETWORK:-localhost}"

if [ "$polkadot_network" == "westend" ]; then
  # Todo: There is no penpal nodes on westend yet
  subxt codegen --url wss://westend-bridge-hub-rpc.polkadot.io >src/parachains/bridgehub.rs
  subxt codegen --url wss://westend-asset-hub-rpc.polkadot.io >src/parachains/assethub.rs
  subxt codegen --url wss://westend-rpc.polkadot.io >src/parachains/relaychain.rs
  subxt codegen --url wss://westend-penpal-rpc.polkadot.io >src/parachains/penpal.rs
else
  if ! lsof -Pi :11144 -sTCP:LISTEN -t >/dev/null; then
      echo "substrate nodes not running, please start with the e2e setup and rerun this script"
      exit 1
  fi
  # Fetch metadata from BridgeHub and generate client
  subxt codegen --url ws://localhost:11144 >src/parachains/bridgehub.rs \
  --derive ::subxt::ext::subxt_core::ext::codec::Encode \
  --derive ::subxt::ext::subxt_core::ext::codec::Decode
  subxt codegen --url ws://localhost:12144 > src/parachains/assethub.rs \
  --derive-for-type staging_xcm::v5::location::Location=Clone,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Encode,recursive \
  --derive-for-type staging_xcm::v5::location::Location=::subxt::ext::codec::Decode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Encode,recursive \
  --derive-for-type xcm::VersionedXcm=::subxt::ext::codec::Decode,recursive \
  --derive-for-type staging_xcm::v5::asset::AssetId=Clone,recursive \
  --derive-for-type staging_xcm::v5::asset::Assets=Clone,recursive
  subxt codegen --url ws://localhost:9944 >src/parachains/relaychain.rs \
  --derive ::subxt::ext::subxt_core::ext::codec::Encode \
  --derive ::subxt::ext::subxt_core::ext::codec::Decode
  subxt codegen --url ws://localhost:13144 >src/parachains/penpal.rs \
  --derive-for-type staging_xcm::v5::location::Location=Clone,recursive \
  --derive-for-type staging_xcm::v5::asset::AssetId=Clone,recursive \
  --derive-for-type staging_xcm::v5::asset::Assets=Clone,recursive \
  --derive ::subxt::ext::subxt_core::ext::codec::Encode \
  --derive ::subxt::ext::subxt_core::ext::codec::Decode

fi


