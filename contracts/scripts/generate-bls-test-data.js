#!/usr/bin/env node

/**
 * Generate BLS12-381 test data for Solidity tests
 *
 * This script generates valid BLS signatures that can be verified
 * by the BLS12381.sol library using Ethereum's EIP-2537 precompiles.
 */

const { bls12_381 } = require('@noble/curves/bls12-381');
const { sha256 } = require('@noble/hashes/sha256');
const { bytesToHex, hexToBytes } = require('@noble/hashes/utils');

// Generate 3 validator keypairs
console.log('Generating BLS12-381 test data...\n');

const validators = [];
for (let i = 0; i < 3; i++) {
    // Generate random private key
    const privateKey = bls12_381.utils.randomPrivateKey();

    // Derive public key (G2 point)
    const publicKey = bls12_381.getPublicKey(privateKey);

    validators.push({
        index: i,
        privateKey: bytesToHex(privateKey),
        publicKey: bytesToHex(publicKey)
    });

    console.log(`Validator ${i}:`);
    console.log(`  Private Key: 0x${bytesToHex(privateKey)}`);
    console.log(`  Public Key (${publicKey.length} bytes): 0x${bytesToHex(publicKey)}`);
    console.log();
}

// Create a test message (commitment hash)
const message = sha256('test-commitment-message');
console.log(`Message (commitment hash): 0x${bytesToHex(message)}`);
console.log();

// Sign the message with each validator
const signatures = [];
for (let i = 0; i < validators.length; i++) {
    const signature = bls12_381.sign(message, hexToBytes(validators[i].privateKey));
    signatures.push(signature);

    console.log(`Validator ${i} Signature (${signature.length} bytes): 0x${bytesToHex(signature)}`);

    // Verify individual signature
    const isValid = bls12_381.verify(signature, message, hexToBytes(validators[i].publicKey));
    console.log(`  Individual verification: ${isValid ? '✓ VALID' : '✗ INVALID'}`);
    console.log();
}

// Aggregate signatures
const aggregatedSignature = bls12_381.aggregateSignatures(signatures);
console.log(`Aggregated Signature (${aggregatedSignature.length} bytes): 0x${bytesToHex(aggregatedSignature)}`);
console.log();

// Verify aggregated signature
const publicKeys = validators.map(v => hexToBytes(v.publicKey));
const isAggregatedValid = bls12_381.verifyBatch(
    aggregatedSignature,
    message,
    publicKeys
);
console.log(`Aggregated verification: ${isAggregatedValid ? '✓ VALID' : '✗ INVALID'}`);
console.log();

// Generate Solidity test data
console.log('='.repeat(80));
console.log('Solidity Test Data:');
console.log('='.repeat(80));
console.log();

console.log('// Message (commitment hash)');
console.log(`bytes32 constant TEST_MESSAGE = 0x${bytesToHex(message)};`);
console.log();

console.log('// Validator public keys (G2 points, 96 bytes each)');
validators.forEach((v, i) => {
    console.log(`bytes constant TEST_PUBKEY_${i} = hex"${v.publicKey}";`);
});
console.log();

console.log('// Individual signatures (G1 points, 48 bytes each)');
signatures.forEach((sig, i) => {
    console.log(`bytes constant TEST_SIGNATURE_${i} = hex"${bytesToHex(sig)}";`);
});
console.log();

console.log('// Aggregated signature (G1 point, 48 bytes)');
console.log(`bytes constant TEST_AGGREGATED_SIGNATURE = hex"${bytesToHex(aggregatedSignature)}";`);
console.log();

console.log('='.repeat(80));
console.log('Note: These signatures are cryptographically valid and can be verified');
console.log('using the BLS12381.sol library with EIP-2537 precompiles.');
console.log('='.repeat(80));
