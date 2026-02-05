use sp1_sdk::{utils, HashableKey, ProverClient, SP1Stdin};
use std::fs;

use serde::{Deserialize, Serialize};

// Types matching the program inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub block_number: u32,
    pub validator_set_id: u64,
    pub payload: Vec<PayloadItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadItem {
    pub payload_id: [u8; 2],
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorProof {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub index: u32,
    pub account: [u8; 20],
    pub proof: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMRLeaf {
    pub version: u8,
    pub parent_number: u32,
    pub parent_hash: [u8; 32],
    pub next_authority_set_id: u64,
    pub next_authority_set_len: u32,
    pub next_authority_set_root: [u8; 32],
    pub parachain_heads_root: [u8; 32],
}

/// The ELF of the program.
pub const BEEFY_CLIENT_ELF: &[u8] = include_bytes!(
    "../../program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1-beefy-client"
);

fn main() -> anyhow::Result<()> {
    utils::setup_logger();
    let client = ProverClient::from_env();
    let (pk, vk) = client.setup(BEEFY_CLIENT_ELF);
    println!("Generated verification key: {}", vk.bytes32());

    // Create the stdin for the program
    let mut stdin = SP1Stdin::new();

    // Read command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} submit-fiat-shamir <commitment.json> <bitfield.json> <proofs.json> <leaf.json> <leaf_proof.json> <leaf_proof_order.json>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "submit-fiat-shamir" => {
            if args.len() < 8 {
                eprintln!("Usage: {} submit-fiat-shamir <commitment.json> <bitfield.json> <proofs.json> <leaf.json> <leaf_proof.json> <leaf_proof_order.json>", args[0]);
                std::process::exit(1);
            }

            // Read and write commitment
            let commitment_json = fs::read_to_string(&args[2])?;
            let commitment: Commitment = serde_json::from_str(&commitment_json)?;
            stdin.write(&commitment);

            // Read and write bitfield
            let bitfield_json = fs::read_to_string(&args[3])?;
            let bitfield: Vec<u64> = serde_json::from_str(&bitfield_json)?;
            stdin.write(&bitfield);

            // Read and write proofs
            let proofs_json = fs::read_to_string(&args[4])?;
            let proofs: Vec<ValidatorProof> = serde_json::from_str(&proofs_json)?;
            stdin.write(&proofs);

            // Read and write leaf
            let leaf_json = fs::read_to_string(&args[5])?;
            let leaf: MMRLeaf = serde_json::from_str(&leaf_json)?;
            stdin.write(&leaf);

            // Read and write leaf proof
            let leaf_proof_json = fs::read_to_string(&args[6])?;
            let leaf_proof: Vec<[u8; 32]> = serde_json::from_str(&leaf_proof_json)?;
            stdin.write(&leaf_proof);

            // Read and write leaf proof order
            let leaf_proof_order_json = fs::read_to_string(&args[7])?;
            let leaf_proof_order: Vec<u8> = serde_json::from_str(&leaf_proof_order_json)?;
            stdin.write(&leaf_proof_order);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }

    // Execute the program
    let proof = client.prove(&pk, &stdin).run()?;

    // Verify the proof
    client.verify(&proof, &vk)?;

    println!("Proof verified successfully!");
    println!("Public values: {:?}", proof.public_values);

    // Save proof and public values for on-chain verification
    fs::write("proof.bin", hex::encode(proof.bytes()))?;
    fs::write(
        "public_values.json",
        serde_json::to_string_pretty(&proof.public_values)?,
    )?;

    Ok(())
}
