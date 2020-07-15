# Ethereum Transaction Prover

The prover library will be used by a trusted operator of the relayer service to sign Ethereum transaction data with a Polkadot-compatible signature that can be provided as proof of a valid Ethereum transaction on Polkadot.


## How it works

Ethereum contains a copy of the state trie root in every block. The state trie is a Patricia-Merkle trie which enables someone to specify the contents of an account and prove it by giving some hashes for a few internal nodes of the trie. With just a Merkle path from the transaction to the root node, anyone can verify the root of the state trie with therefore validate that the transaction is valid. This cryptographic verification mechanism is called Simple Payment Verification (SPV).

Running as a light client, SPVs enable the relayer to securely know the contents of a transaction (including token transfers or other arbitrary data) as of a specified block just as well as a full node would be able to. By appropriately packaging the transaction data and using a Polkadot-compatible signature, each transaction will be verifiable on Substrate chains.

## Status

The prover's types and interfaces are defined with core functionality under development.

