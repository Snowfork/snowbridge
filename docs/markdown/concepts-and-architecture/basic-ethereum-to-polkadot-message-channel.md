---
layout: default
title: Basic Ethereum to Polkadot Message Channel
nav_order: 5
permalink: /concepts/basic-ethereum-to-polkadot-message-channel
parent: Concepts and Architecture
---

# Basic Ethereum to Polkadot Message Channel

## Overview
The Basic Ethereum to Polkadot Message Channel manages a channel for sending Ethereum RPCs out from Ethereum to Polkadot via events. It consists of a smart contract on the Ethereum side and a corresponding pallet on the parachain side. 

The smart contract is responsible for accepting requests from other smart contracts for Ethereum RPCs to be sent over to Polkadot. It uses Ethereum Events as the medium for incoming messages, so once accepting a request, it adds a nonce and emits an event that a corresponding pallet will receive on the Polkadot side.

The corresponsing receiving pallet accepts as input a set of events and a transaction receipt proof, calls out to the [Ethereum Light Client Verifier](./ethereum-verifier) to verify that an Ethereum event is in fact valid and included in the Ethereum chain, and then it processes that event by forwarding it to a destination pallet.

This channel is intended to be a simple straightforward channel which provides only a basic guarantee of deliverability and replay protection.

## Motivation
The intention behind a channel with minimal guarantees is for it to be used as a basic bootstrap mechanism for other more complicated channels. For example, other more complex channel designs may want to depend on already-bootstrapped cross-chain applications, like PolkaETH or SnowDOT for their incentives, so this channel can be used to bootstrap liquidity in those applications.

A channel used for bootstrapping must at the very least provide a basic guarantee of deliverability and of replay protection to ensure correctness and censorship resistance, but no other guarantees are required. It must also only depend on the core Light Client Verifier pallet and no others.

## Replay Protection
It uses an increasing integer as a nonce for replay protection when sending messages, and tracks all nonces that have been used in the past for replay protection when receiving messages.

Messages can be processed in any order. The receiving pallet stores the set of nonces that have been used so that it can check for replay protection. Whenever a new message is processed, that message's nonce is added to the set.

For V1, a basic key value store will be used for storing nonces. This is ofcourse not optimized as it requires storing a new item for every message, so longer term a more optimized alternative can be implemented.

## Deliverability
The receiving pallet places no constraints on accepting messages other than that they can be verified by the Ethereum Verifier and that their nonce is unused. Users are expected to deliver their own messages themselves without depending on any third party.

Once a message has been delivered and checked, it is routed to the destination pallet specified in the EthereumRPC.

## Interface
The channel smart contract accepts Ethereum RPCs and adds a nonce, outputting events with the following format:

```
BasicE2PChannelEvent {
    nonce Int
    rpc EthereumRPC
}
```

## Optimizations
Future optimizations could be done to the replay protection mechanism to improve the storage costs of this channel. Some examples to be considered are:
 - Using a small proof stored on-chain to summarize the set of nonces that have been used, like a merkle tree root that can quickly verify whether a given nonce has been used and can be updated with new nonces as they are used
 - Using something like a RSA accumulator

 Alternatively, a more complex design for the bootstrap channel could be explored. For example, a channel that used an increasing nonce but then had a bidding market for acceptance of messages that are then processed in order every n blocks would only require storing the number of processed messages to be checked for replay protection rather than storing every nonce.