---
layout: default
title: MMR Light-Client for Polkadot Relay Chain
nav_order: 3
permalink: /concepts/polkadot-light-client-verifier/mmr-light-client
parent: Polkadot Light Client Verifier
grand_parent: Concepts and Architecture
---
# MMR Light-Client for Polkadot Relay Chain

After the interactive protocol runs, we have new [BEEFY](https://github.com/paritytech/grandpa-bridge-gadget) MMR commitments. These are the root hashes of merkle mountain ranges that contain data for updates to the Polkadot Validator set and data for new relay chain headers.

We use merkle proofs to verify the contents of each BEEFY MMR, extracting the above two kinds of data.

New validator set updates are applied to our interactive protocol client so that it knows which validator signatures will be valid for the next block.

The new Parachain header for our Snowbridge chain is verified and then provided to our [Parachain Light Client](/concepts/polkadot-light-client-verifier/parachain-light-client) to be used for verifying and processing new bridge messages.