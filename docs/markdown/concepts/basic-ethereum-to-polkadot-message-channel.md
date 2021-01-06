---
layout: default
title: Basic Message Channel (Ethereum → Polkadot)
nav_order: 6
permalink: /concepts/basic-ethereum-to-polkadot-message-channel
parent: Concepts and Architecture
---
# Basic Message Channel (Ethereum → Polkadot)

## Overview

This basic channel provides a mechanism for sending messages out from Ethereum to Polkadot via events. It consists of a smart contract on the Ethereum side and a corresponding pallet on the parachain side.

The smart contract is responsible for accepting requests from other smart contracts for Polkadot RPCs to be sent over to Polkadot. It uses Ethereum Events as the medium for sending messages, so after accepting a request, it adds a nonce and emits an event that the corresponding pallet will receive on the Polkadot side.

The corresponsing receiving pallet accepts as input a set of events and a transaction receipt proof, calls out to the [Ethereum Light Client Verifier](./ethereum-verifier) to confirm that the containing block is considered final by having a predefined number of confirmations and to verify that each Ethereum event is in fact valid and included in the Ethereum chain, and then it processes those events by forwarding them to their destination pallet.

This channel is intended to be a simple straightforward channel which provides only a basic guarantee of deliverability and replay protection.

## Motivation

The intention behind a channel with minimal guarantees is for it to be used as a basic bootstrap mechanism for other more complicated channels. For example, other more complex channel designs may want to depend on already-bootstrapped cross-chain applications, like SnowETH or SnowDOT for their incentives, so this channel can be used to bootstrap liquidity in those applications.

A channel used for bootstrapping must at the very least provide a basic guarantee of deliverability and of replay protection to ensure correctness and censorship resistance, but no other guarantees are required. It must also only depend on the core bridge pallets and no bridge applications.

## Replay Protection

This basic channel uses an increasing integer as a nonce for replay protection when sending messages. It tracks all nonces that have been used in the past, and checks against them for replay protection when receiving messages.

Messages can be processed in any order. The receiving pallet stores the set of nonces that have been used so that it can check for replay protection. Whenever a new message is processed, that message's nonce is added to the set.

For V1, a basic key-value store will be used for storing nonces. This is of course not optimized as it requires storing a new item for every message, so longer term a more optimized alternative can be implemented.

## Deliverability

The receiving pallet places no constraints on accepting messages other than that they can be verified by the Ethereum Verifier and that their nonce is unused. Users are expected to deliver their own messages themselves without depending on any third party.

Once a message has been delivered and checked, it is routed to the destination pallet specified in the PolkadotRPC.

## Interface

The channel smart contract accepts Polkadot RPCs and adds a nonce, outputting events with the following format:

```
ChannelEvent {
    nonce Int
    rpc PolkadotRPC
}
```

## Optimizations

Future optimizations could be done to the replay protection mechanism to improve the storage costs of this channel. Some examples to be considered are:

- Using a small proof stored on-chain to summarize the set of nonces that have been used, like a merkle tree root that can quickly verify whether a given nonce has been used and can be updated with new nonces as they are used
- Exploring something like a RSA accumulator or Bloom Filter

In the future, a more complex design for the bootstrap channel could be explored. For example, a channel that batches messages into blocks at regular intervals would be able to process messages in order and so would only require storing the nonce of the most recently processed message to be checked for replay protection rather than storing every processed nonce.

Adding something like a bidding market for ordering and inclusion of messages within a block could be valuable too, though this would then mean having a more nuanced guarantee of message deliverability which would be dependent on the bidding market, so is not ideal in the short term for our initial basic bootstrap implementation.
