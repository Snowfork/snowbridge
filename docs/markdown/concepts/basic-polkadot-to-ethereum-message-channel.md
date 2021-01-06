---
layout: default
title: Basic Message Channel (Polkadot → Ethereum)
nav_order: 7
permalink: /concepts/basic-polkadot-to-ethereum-message-channel
parent: Concepts and Architecture
---
# Basic Message Channel (Polkadot → Ethereum)

## Overview

This basic channel provides a mechanism for sending messages out from Polkadot to Ethereum via Snowbridge Parachain commitments. It consists of our Snowbridge Parachain and its Commitments pallet, which operates a basic channel that provides replay protection.

The Commitments pallet is responsible for accepting requests from other pallets or parachains via XCMP for Ethereum RPCs to be sent over to Ethereum. After accepting a request, it adds a nonce and puts the message into a queue. At a fixed interval (initially once a minute) it produces a commitment to that queue in the form of a single hash of all the messages in that queue that is added to the parachain header.

The corresponding receiving Snowbridge Parachain Light Client smart contract accepts as input this commitment as well as the set of messages in that commitment. It validates the inclusion of the commitment in the MMR as described [here](./ethereum-light-client-verifier) and then verifies that those messages are included in the commitment by hashing them and then processes them in order by calling out to the Ethereum smart contracts for each Ethereum RPC.

This channel is intended to be a simple straightforward channel which provides only a basic guarantee of deliverability and replay protection. As a side effect of the queue system, messages are ordered and so this basic channel provides strict ordering too. In future, the censorship resistance of the basic bridge will be improved even further - see [this issue](https://github.com/Snowfork/polkadot-ethereum/issues/196){: target="\_blank"} for more details.

## Replay Protection

This basic channel uses an ordered queue of commitments with ordered messages. Messages and commitments must be processed in order. It stores the nonce of the most recently processed message, which is a strictly increasing integer, and so will only play new commitments/messages and update the count once completed.

## Deliverability

The receiving smart contract places no constraints on accepting messages other than that they can be verified by the Polkadot Light Client Verifier and that they are new commitments with a higher nonce. Users are expected to deliver commitments to their own messages as well as all messages in those commitments themselves without depending on any third party. Given that the bridge is ordered, users are also responsible for ensuring the bridge is up to date with no blockage, though this constraint will be removed in a more optimized future version described [here](https://github.com/Snowfork/polkadot-ethereum/issues/196){: target="\_blank"}.

Once a commitment and its messages have been delivered and checked, they are routed to the destination smart contract specified in the EthereumRPC.

## Other

Further details will be added later, including ideas around:

- The implications of ordered messaging with users that may attempt to block the queue
- The hash and data structure used for commitments and alternatives that support verification without needing all messages
- Batch processing of messages
- Maximum gas constraints per message and per commitment for deliverability guarantees
- Attack vectors given that commitments can contain messages across multiple users which must be played in order
