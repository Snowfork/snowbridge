---
layout: default
title: Polkadot Light Client Verifier
nav_order: 5
permalink: /concepts/polkadot-light-client-verifier/
parent: Concepts and Architecture
has_children: true
---

# Polkadot Light Client Verifier

# Overview

The Polkadot Light Client Verifier for the bridge is a collection of components that allow for trustless verification of bridge messages that come from our bridge parachain on Polkadot and that are sent to Ethereum. This means that it covers trustless verification of:

- Polkadot state, ie, changes to the Polkadot Relay Chain which mark new finalized blocks on our bridge parachain
- Our parachain state, ie, new parachain blocks
- The bridge messages themself, ie, messages that users/applications submit to our bridge parachain for trustless relay

It also needs to do this all while being cheap enough in terms of Ethereum gas prices in order to remain sustainable. There are various components that fit together to make this happen.

## Proof Creation

We create various custom proofs that will be used for verification on Ethereum.

### Polkadot Relay Chain Proofs

To facilitate our bridge, the Polkadot relay chain will have a new gadget, BEEFY, added that will produce merkle mountain range (MMR) roots signed by Polkadot validators that commit to both new Polkadot Relay chain blocks and new parachain blocks in a form that is cheaply verifiable on Ethereum. Producing these commitments will be mandatory for Polkadot validators, and will have additional new slashing conditions associated with them, and so will ensure that verification of and trust in these custom commitments is as strong as direct verification of Polkadot Consensus.

Following these commitments will effectively allow us to follow both (1) new Polkadot Relay chain block headers and (2) new Parachain block headers.

For more details, see [BEEFY](https://github.com/paritytech/grandpa-bridge-gadget){: target="\_blank"}

### Snowbridge Parachain Proofs

Our parachain will have a custom commitment scheme for committing to bridge messages on our parachain and placing those commitments into our parachain header along with a custom light client for our parachain that runs on Ethereum.

For more details, see [Parachain Commitments](/concepts/polkadot-light-client-verifier/parachain-commitments)

## Proof Verification

Proof verification happens in Solidity and has 3 main steps.

### Following the Polkadot Relay Chain

The first step for trustless verification of our bridge on Ethereum starts with following the Polkadot relay chain via following new BEEFY MMR roots _(as mentioned above)_ as they are produced and verifying their validity. Their validity is verified by checking that they are signed by the correct set of Polkadot validators.

For more details, see [Polkadot Relay Chain Interactive Update Protocol](/concepts/polkadot-light-client-verifier/interactive-protocol)

### Applying New Relay Chain MMR UPdates
These verified relay chain MMR updates contain validator set updates and parachain header updates. We can use them to update our knowledge about Polkadot validators and to extract and follow new headers of our Snowbridge parachain blocks.

For more details, see [Polkadot Relay Chain MMR Light Client](/concepts/polkadot-light-client-verifier/mmr-light-client)

### Verifying Bridge Messages

Lastly, with these verified parachain blocks, we have a custom Snowfork Parachain light client that uses our Parachain Commitments to verify individual bridge messages.

For more details, see [Parachain Light Client](/concepts/polkadot-light-client-verifier/parachain-light-client)
