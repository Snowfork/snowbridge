#!/usr/bin/env bash

set -eu

mkdir -p src/contracts

subxt codegen --url http://localhost:11144 | rustfmt --edition 2021 --emit=stdout > src/parachains/bridgehub.rs
forge bind --module --overwrite --bindings-path src/contracts --root ../core/packages/contracts
