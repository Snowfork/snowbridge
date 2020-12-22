---
layout: default
title: Ethereum Light Client Verifier
nav_order: 3
permalink: /concepts/ethereum-light-client-verifier
parent: Concepts and Architecture
---
# Ethereum Light Client Verifier

This is a Rust pallet that is responsible for watching and verifying Ethereum Mainnet. It runs a light client that can track Ethereum as it progresses. It accepts new Ethereum headers once they are produced, checks proof of work and follows the longest chain. It also handles short-term forks up to some number of confirmations. It is also responsible for verifying actual messages that are coming across the bridge from Ethereum smart contract applications.

## Proof of Work Verification

The proof of work verification is done based on Ethash verification as described in https://eth.wiki/en/concepts/ethash/ethash. One caveat is that typical light clients will store the full cache used to generate the hashing dataset. On our parachain, even the cache is too big (~64MB+ at the moment), so instead of storing it directly, we store a merkle root that commits to 2 - 4 years worth of cache data. As part of the proof of work verification, the relayer must submit a merkle proof of the data used for generating the proof of work on each block.

This commitment will need to be updated every 2+ years, and is expected to be done so via decentralized governance which will be implemented post-launch.

## Transaction Receipt proofs

The verifier needs to also verify messages from Ethereum smart contracts. To do so, we verify Ethereum Transaction Receipt proofs which contain the event logs of all events in the Ethereum transaction. Transaction Receipts also include additional data, for example transaction input data, but we primarily use the event data as our medium for bridge messages and so only extract event data for now.
