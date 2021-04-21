---
layout: default
title: Relayer Architecture
parent: Validating and Relaying
permalink: /relayer-architecture/
nav_order: 3
---

# Relayer Architecture
The relayer is a single Go binary that acts as a container for various workers that are involved in the relaying process. As our bridge requires a mix of different kinds of data types and proofs to be relayed at different times, co-ordinated in different ways, we split up these responsibilities into different workers that separate concerns.

## Worker Goals
 - Each worker is expected to be keep its state entirely in memory and not use any local file system or stateful storage other than its own memory (and of course, the on-chain state)
 - All transactions sent by workers should be idempotent (the on-chain logic does guarantee this in most scenarios, so usually the worker does not need to worry about this, but in the BEEFY relay process there may be some considerations)
 - Workers should not communicate with eachother directly. If they have any dependency on eachother's work, they should co-ordinate via [Stigmergy](https://en.wikipedia.org/wiki/Stigmergy) through querying on-chain data.
 - Workers should recover gracefully in case of failures. This means that they should shut down cleanly upon any error, deadlocks should never happen. On startup, they should refresh their knowledge of the state of the world through on-chain queries, resume from that state, and catch themselves and the on-chain state up to date before any other behaviour.
- Each worker can be run independent of other workers in the binary.

# Current Workers

## Ethereum -> Parachain Feeder
This worker is responsible for feeding the parachain with all up to date state from Ethereum. It is currently responsible for relaying new Ethereum headers, proof-of-work data, as well as channel and message data to the parachain. In future this could be split up into a seperate header/proof-of-work worker and channel/message worker, but for now they're bundled together.

## Relay Chain -> Ethereum Feeder
This worker is responsible for keeping the Ethereum-side Relay Chain Light Client up to date. It is responsible for taking BEEFY signatures/commitments and mmr-proofs from the relay chain and submitting the to Ethereum via the 2-step commit/complete process.

## Parachain -> Ethereum Feeder
This worker is responsible for keeping the Ethereum-side channels/Parachain Light Clients up to date. It is responsible for taking parachain messages, parachain commitments, parachain-block-header-proofs and mmr-proofs from both the parachain and relay chain and submitting them to Ethereum.

# Shared Interface
The relayer as a worker container starts up each worker with a shared interface for common functionality. It runs as a single Go process with threads for each worker. The interface provides the following shared functionality:
 - metrics/monitoring
 - logging
 - config
 - panic-detection and recovery
 - error-detection and recovery
 - completion and restart
