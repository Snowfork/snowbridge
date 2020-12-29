---
layout: default
title: Basic Polkadot to Ethereum Message Channel
nav_order: 7
permalink: /concepts/basic-polkadot-to-ethereum-message-channel
parent: Concepts and Architecture
---

# Basic Polkadot to Ethereum Message Channel

## Overview

The Basic Polkadot to Ethereum Message Channel manages a channel for sending Ethereum RPCs out from Polkadot to Ethereum via Snowbridge Parachain commitments. It consists of our Snowbridge Parachain and its Commitments pallet, which operates a basic channel that provides replay protection.

The Commitments pallet is responsible for accepting requests from other pallets or parachains via XCMP for Ethereum RPCs to be sent over to Ethereum. After accepting a request, it adds a nonce and puts the message into a queue. At a fixed interval (initially once a minute) it produces a commitment to that queue in the form of a single hash of all the messages in that queue that is added to the parachain header.

The corresponding receiving Snowbridge Parachain Light Client smart contract accepts as input this commitment as well as the set of messages in that commitment. It validates the inclusion of the commitment in the MMR as described [here](./ethereum-light-client-verifier) and then verifies that those messages are included in the commitment by hashing them and then processes them in order by calling out to the Ethereum smart contracts for each Ethereum RPC.

<!-- TODO: I believe the basic bridge should use merkle tree based commitments to ensure there can be no cencorship. The goal is not to optimize for costs, but to minimize trust/attack vectors at the expense of a higher cost. We don't need orderding for that. The approach described here can be attacked, e. g. by flooding with txs during high gas price times. -->

This channel is intended to be a simple straightforward channel which provides only a basic guarantee of deliverability and replay protection. As a side effect of the queue system, messages are ordered and so this basic channel provides strict ordering too.

## Replay Protection

This basic channel uses an ordered queue of commitments with ordered messages. Messages and commitments must be processed in order. It stores the count of the most recently processed commitment/message, and so will only play new commitments/messages and update the count once completed.

<!-- TODO: What do we mean by "count" here? A strictly increasing nonce per commitment? Per message? -->

## Deliverability

The receiving smart contract places no constraints on accepting messages other than that they can be verified by the Polkadot Light Client Verifier and that they are new commitments with a higher nonce. Users are expected to deliver commitments to their own messages as well as all messages in those commitments themselves without depending on any third party.

<!-- TODO: this is an issue, expecting users to process their and all preceeding messages will be a big hurdle. If there is no mechanism to pay a relayer to process all, this will lead to a [Tragedy of the commons](https://en.wikipedia.org/wiki/Tragedy_of_the_commons) -->

Once a commitment and its messages have been delivered and checked, they are routed to the destination smart contract specified in the EthereumRPC.

## Other

Further details will be added later, including ideas around:

- The implications of ordered messaging with users that may attempt to block the queue
- The hash and data structure used for commitments and alternatives that support verification without needing all messages
- Batch processing of messages
- Maximum gas constraints per message and per commitment for deliverability guarantees
- Attack vectors given that commitments can contain messages across multiple users which must be played in order
