#!/bin/bash

# Build the ZK program
echo "Building SP1 Beefy Client program..."
cd program
cargo prove build --release

# Generate the verification key
echo "Generating verification key..."
cargo prove vkey --elf target/riscv32im-succinct-zkvm-elf/release/sp1-beefy-client > ../contracts/vkey.txt

# Build the prover script
echo "Building prover script..."
cd ../script
cargo build --release

echo "Build complete!"
echo ""
echo "To run a proof:"
echo "  cd ../script"
echo "  cargo run -- <command> <arguments>"
echo ""
echo "To verify on-chain:"
echo "  Deploy contracts/src/SP1BeefyClient.sol with the verification key from contracts/vkey.txt"