#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

pushd $root_dir/smoketest

./make-bindings.sh
echo "build tests"
cargo test --no-run
echo "run register token test"
cargo test --test register_token --release -- --nocapture
echo "run send token test"
cargo test --test send_token --release -- --nocapture
echo "run transfer token test"
cargo test --test transfer_token --release -- --nocapture
echo "run send token to penpal test"
cargo test --test send_token_to_penpal --release -- --nocapture

popd
