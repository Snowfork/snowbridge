#!/bin/bash

# Build the ZK program
echo "Building SP1 Beefy Client program..."
cd program
cargo prove build

# Generate the verification key
echo "Generating verification key..."
ELF_PATH="target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1-beefy-client"
if [ ! -f "$ELF_PATH" ]; then
	echo "ELF not found at $ELF_PATH"
	echo "Run 'cargo prove build' successfully before generating vkey."
	exit 1
fi
cargo prove vkey --elf "$ELF_PATH" > ../contracts/vkey.txt

# Build the prover script
echo "Building prover script..."
cd ../script
cargo build --release

echo "Build complete!"
echo ""
echo "To run a proof:"
echo "  cd ../script"
echo "  cargo run -- submit-fiat-shamir <commitment.json> <bitfield.json> <proofs.json> <leaf.json> <leaf_proof.json> <leaf_proof_order.json>"
echo ""
echo "To verify on-chain:"
echo "  Deploy contracts/src/SP1BeefyClient.sol with the verification key from contracts/vkey.txt"
