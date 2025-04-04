#!/usr/bin/env bash
set -eu

source scripts/set-env.sh

pushd $root_dir/smoketest

./make-bindings.sh
echo "build tests"
cargo test --no-run
echo "run register ENA v2"
cargo test --test v2 --release -- --test register_ena --nocapture
echo "run register PNA v2"
cargo test --test v2 --release -- --test register_pna --nocapture
echo "run send ENA v1"
cargo test --test send_token --release -- --nocapture
echo "run transfer ENA v1"
cargo test --test transfer_token --release -- --nocapture
echo "run send ENA to penpal v1"
cargo test --test send_token_to_penpal --release -- --nocapture
echo "run send PNA v1"
cargo test --test transfer_polkadot_token --release -- --nocapture
echo "run transfer PNA v1"
cargo test --test send_polkadot_token --release -- --nocapture
echo "run send ENA v2"
cargo test --test v2 --release -- --test send_ena_to_ah --nocapture
echo "run send ENA to penpal v2"
cargo test --test v2 --release -- --test send_ena_to_penal --nocapture
echo "run transfer ENA v2"
cargo test --test v2 --release -- --test transfer_ena --nocapture
echo "run send PNA v2"
cargo test --test v2 --release -- --test send_pna --nocapture
echo "run transfer PNA v2"
cargo test --test v2 --release -- --test transfer_pna --nocapture
popd
