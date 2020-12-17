---
layout: default
title: Basic Polkadot to Ethereum Message Channel
nav_order: 6
permalink: /concepts/basic-polkadot-to-ethereum-message-channel
parent: Concepts and Architecture
---
# Basic Polkadot to Ethereum Message Channel

## Overview
The Basic Polkadot to Ethereum Message Channel manages a channel for sending Ethereum RPCs out from Polkadot to Ethereum via Snowbridge Parachain commitments. It consists of our Snowbridge Parachain and its Commitments pallet, which operates a basic channel that provides replay protection.

The Commitments pallet is responsible for accepting requests from other pallets or parachains via XCMP for Ethereum RPCs to be sent over to Ethereum. After accepting a request, it adds that message to a queue and once a minute produces a commitment to that queue in the form of a single hash of all the messages in that queue that is added to the parachain header.

The corresponsing receiving Snowbridge Parachain Light Client smart contract accepts as input this commitment once it has been verified, as well as the set of messages in that commitment. It then verifies that those messages are included in the commitment by hashing them and then processes them in order by calling out to the Ethereum smart contracts for each Ethereum RPC.

This channel is intended to be a simple straightforward channel which provides only a basic guarantee of deliverability and replay protection. As a side effect of the queue system, messages are ordered and so this basic channel provides strict ordering too.

## Replay Protection
This basic channel uses an ordered queue of commitments with ordered messages. Messages and commitments must be processed in order. It stores the count of the most recently processed commitment/message, and so will only play new commitments/messages and update the count once completed.

## Deliverability
The receiving smart contract places no constraints on accepting messages other than that they can be verified by the Polkadot Light Client Verifier and that they are new commitments with a higher nonce. Users are expected to deliver commitments to their own messages as well as all messages in those commitments themselves without depending on any third party.

Once a commitment and its messages have been delivered and checked, they are routed to the destination smart contract specified in the EthereumRPC.

## Interface
The Commitments pallet contract accepts Ethereum RPCs and adds them to the queue, producting a commitment every minute.

## Other
Further details will be added later, including ideas around:
- The implications of ordered messaging with users that may attempt to block the queue
- The hash and data structure used for commitments and alternatives that support verification without needing all messages
- Batch processing of messages
- Maximum gas constraints per message and per commitment for deliverability guarantees
- Attack vectors given that commitments can contain messages across multiple users which must be played in order