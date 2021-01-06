---
layout: default
title: Snowbridge Parachain
nav_order: 4
permalink: /concepts/snowbridge-parachain
parent: Concepts and Architecture
---

# Snowbridge Parachain

The Snowbridge parachain is a tokenless, permissionless parachain. For now, anyone can run a collator node. The parachain facilitates the following functionality:

- Allowing other applications on other parachains to send and receive messages from smart contract applications on Ethereum
- Wrapping ETH on Ethereum into a Polkadot-based asset via SnowETH
- Wrapping ERC20 tokens on Ethereum into Polkadot-based assets via SnowERC20
- Wrapping DOTs into Etherum-based assets via SnowDOT

## Block production and permissioning

As mentioned, our parachain is permissionless and tokenless, so anyone can run a collator node. This is somewhat censorship-resistant, but it does mean that an attacker could DoS the bridge by running nodes that submit empty blocks. Eventually honest blocks are likely to go through, but we do plan to mitigate this in future. Currently Cumulus/Polkadot do not easily support alternative block production and permissioning mechanisms, but we are likely to move to proof-of-stake based consensus with stake being placed in DOT tokens once it is supported by Cumulus.

## Parachain Pallets

The parachain provides low level bridging to Ethereum, but also contains pallets for a few high level applications for ETH, DOT and ERC20 pegged assets. These core bridge applications are detailed further here: [SnowETH](../core-applications/snoweth), [SnowERC20](../core-applications/snowerc20), [SnowDOT](../core-applications/snowdot)

We may add additional application pallets in future, for example for NFT/ERC721 bridging support or for other functionality that is likely to be needed across the wider Polkadot ecosystem.
