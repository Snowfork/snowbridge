---
layout: default
title: Building With Snowbridge
nav_order: 6
has_children: true
permalink: /building-with-snowbridge/
---

# Building With Snowbridge

Draft
{: .label .label-yellow }

Snowbridge will support various bits of functionality for blockchain developers.

## Running our Software

If you're interested in running, using or building on any of our software directly, see our [GitHub repository](https://github.com/Snowfork/polkadot-ethereum){: target="\_blank"}.

## Integrating with SnowETH, SnowERC20, or SnowDOT

For Polkadot Parachain developers, Snowbridge will be the chain that facilitates creation of SnowETH, SnowERC20, or SnowDOT. If you want to support or use any of these assets, you can interact via Snowbridge's parachain via XCMP. We will be supporting various XCMP messages for doing so. All our XCMP charges will likely be in DOT, as we have no base currency, so as long as your parachain holds some DOT you should be able to use our XCMP messages permissionlessly.

For more details, see [XCM Interface for Assets](./xcm-for-assets)

On Ethereum, we'll have an ERC20 contract for SnowDOT that will allow for DOT to become part of the Ethereum ecosystem and be used in all Ethereum products that support ERC20.

## Putting your own parachain assets onto Ethereum

If you want to get your own assets onto Ethereum, we'll likely build out support tor that in future. It will also be possible to do that using your own custom cross-chain dApp, as described below.

## Arbitrary state, Cross-Chain Smart Contract Calls and Cross-Chain dApps

Snowbridge supports sending and receiving of arbitrary state between Polkadot and Ethereum, as well as cross-chain smart contract calls. This means it can be used to develop any kinds of cross-chain dApps, not just asset transfer. If you look at the code for our core bridge applications [SnowETH](../core-applications/snoweth), [SnowERC20](../core-applications/snowerc20) and [SnowDOT](../core-applications/snowdot) you'll see that they've been implemented using pairs of Polkadot pallets and Ethereum smart contracts to create cross-chain dApps for those assets via cross-chain smart contract calls.

Unfortunately Snowbridge currently only supports arbitrary state/cross-chain smart contract calls for pallets running on our own Parachain. We're working on building out some lower level XCMP messages that will allow for any pallets and/or smart contracts on any Parachain to be used as part of a cross-chain dApp, but this support is still in progress.

For more details, see [XCM Interface for Arbitrary State and Cross Chain dApps](./xcm-for-state).

We'll also work on tutorials and guides to help parachain and smart contract developers design and build their cross-chain dApps with Snowbridge once XCM is more mature.
