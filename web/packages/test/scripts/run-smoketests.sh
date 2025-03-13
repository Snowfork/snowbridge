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
echo "run register token v2"
cargo test --test register_token_v2 --release -- --nocapture
echo "run send token v2"
cargo test --test send_token_v2 --release -- --nocapture
echo "run send token to penpal v2"
cargo test --test send_token_to_penpal_v2 --release -- --nocapture
popd
