#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

pushd $root_dir/smoketest

./make-bindings.sh
echo "run register token test"
cargo test --test register_token --release -- --nocapture
echo "run send token test"
cargo test --test send_token --release -- --nocapture
echo "run transfer token test"
cargo test --test transfer_token --release -- --nocapture

popd
