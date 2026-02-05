# SP1 Beefy Client

This is a Rust implementation of the BeefyClient contract using the Succinct Labs SP1 zero-knowledge virtual machine (zkVM). The implementation provides a way to generate ZK proofs for BeefyClient operations that can be verified on-chain.

## Architecture

The project consists of three main components:

1. **ZK Program** (`program/`) - A Rust program that runs inside the SP1 zkVM to verify BeefyClient operations
2. **Prover Script** (`script/`) - A script that generates ZK proofs for the various operations
3. **Solidity Verifier** (`contracts/`) - A Solidity contract that verifies the ZK proofs on-chain

## Key Features

1. **Zero-Knowledge Verification**: All BeefyClient verification logic is executed inside the zkVM
2. **Gas Efficiency**: Once proven, verification on-chain is very cheap
3. **Privacy**: Can hide sensitive verification details if needed
4. **Trustlessness**: Anyone can verify the proof without trusting the prover

## Operations Supported

1. **submitInitial**: Begin submission of a new commitment with validator proofs
2. **commitPrevRandao**: Commit to a random seed for validator subsampling
3. **submitFinal**: Complete submission of a commitment with all required signatures
4. **submitFiatShamir**: Submit a commitment using the Fiat-Shamir transformation

## Usage

### Building the ZK Program

```bash
cd program
cargo prove build
```

### Generating Proofs

The script provides several commands for generating proofs:

```bash
# Generate a proof for submitInitial
cd ../script
cargo run -- submit-initial commitment.json bitfield.json proof.json

# Generate a proof for commitPrevRandao
cargo run -- commit-prev-randao <commitment_hash>

# Generate a proof for submitFinal
cargo run -- submit-final commitment.json bitfield.json proofs.json leaf.json leaf_proof.json leaf_proof_order.json

# Generate a proof for submitFiatShamir
cargo run -- submit-fiat-shamir commitment.json bitfield.json proofs.json leaf.json leaf_proof.json leaf_proof_order.json
```

### Verifying On-Chain

1. Deploy the `SP1BeefyClient` contract
2. Use the generated proof to call the appropriate verification function:
   - `verifySubmitInitial` for initial submissions
   - `verifyCommitPrevRandao` for RANDAO commitments
   - `verifySubmitFinal` for final submissions
   - `verifySubmitFiatShamir` for Fiat-Shamir submissions

## Differences from Solidity Implementation

1. **Proof Generation**: Instead of executing verification logic on-chain, it's executed off-chain and proven
2. **Gas Costs**: Much lower on-chain verification costs
3. **Verification Time**: Off-chain computation might take longer but on-chain verification is instant
4. **Trust Model**: Relies on the correctness of the SP1 zkVM instead of EVM execution

## Integration with Existing Infrastructure

This implementation can be integrated with the existing Snowbridge infrastructure by:

1. Replacing the BeefyClient contract with SP1BeefyClient
2. Using the script to generate proofs before submitting to Ethereum
3. Updating relayers to use the new verification flow

## Security Considerations

1. The SP1 zkVM provides cryptographic guarantees of correct execution
2. The Solidity verifier only needs to verify the proof, not re-execute the logic
3. The verification key must be kept up-to-date with the program version
4. The same security assumptions as the original BeefyClient apply to the verification logic