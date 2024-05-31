#!/usr/bin/env bash

set -eux

echo "Checkout polkadot-sdk Snowfork fork"
pushd ..
if [ ! -d "polkadot-sdk" ]; then
  git clone https://github.com/Snowfork/polkadot-sdk.git
fi
pushd  polkadot-sdk
git checkout snowbridge
popd
popd
ln -sf ../polkadot-sdk polkadot-sdk

echo "Checkout lodestar Snowfork fork"
pushd ..
  if [ ! -d "lodestar" ]; then
    git clone https://github.com/ChainSafe/lodestar
  fi
  pushd lodestar
    git checkout $LODESTAR_VERSION
  popd
popd
ln -sf ../lodestar lodestar

echo "Setting up git hooks"
git config --local core.hooksPath hooks/

echo "Installing Rust nightly toolchain"
rustup default stable
rustup target add wasm32-unknown-unknown
rustup install --profile minimal $RUST_NIGHTLY_VERSION
rustup component add --toolchain $RUST_NIGHTLY_VERSION rustfmt
rustup show

echo "Installing sszgen"
go install github.com/ferranbt/fastssz/sszgen@v0.1.3

echo "Installing cargo fuzz"
cargo install cargo-fuzz

echo "Installing web packages"
(cd web && pnpm install)

