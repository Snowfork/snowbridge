#!/usr/bin/env bash

set -eux

echo "Checkout polkadot-sdk"
pushd ..
  if [[ -d polkadot-sdk ]] && (cd polkadot-sdk && git rev-parse --is-inside-work-tree > /dev/null 2>&1); then
     echo "polkadot-sdk already exists"
  else
    repoURL="${POLKADOT_SDK_REPO:-https://github.com/paritytech/polkadot-sdk.git}"

    git clone "$repoURL" polkadot-sdk

    pushd polkadot-sdk
      git pull origin master
    popd
  fi
popd

echo "Checkout lodestar"
pushd ..
  if [ ! -d "lodestar" ]; then
    git clone https://github.com/ChainSafe/lodestar
  fi
  pushd lodestar
    git fetch && git checkout $LODESTAR_VERSION
  popd
popd

echo "Installing Rust nightly toolchain"
rustup default stable
rustup target add wasm32-unknown-unknown
rustup install --profile minimal $RUST_NIGHTLY_VERSION
rustup component add --toolchain $RUST_NIGHTLY_VERSION rustfmt
rustup component add --toolchain stable rust-src rust-analyzer rustfmt
rustup update
rustup show

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@v0.1.3

echo "Installing cargo fuzz"
cargo install cargo-fuzz

echo "Installing web packages"
(cd web && pnpm install)

