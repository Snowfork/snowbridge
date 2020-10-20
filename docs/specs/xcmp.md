---
layout: default
title: XCMP Interface
parent: Specification
nav_order: 2
---

# XCMP Interface <!-- omit in toc -->

_NOTE: work in progress_

- [Introduction](#introduction)
- [Scenarios](#scenarios)
  - [Alice transfers 21 PolkaETH to Bob on another chain](#alice-transfers-21-polkaeth-to-bob-on-another-chain)
  - [Alice attempts to exchange 42 DOT for 21 PolkaETH](#alice-attempts-to-exchange-42-dot-for-21-polkaeth)
- [Asset Identification](#asset-identification)
- [Outstanding Issues](#outstanding-issues)
  - [Numeric precision](#numeric-precision)

## Introduction

Our parachain will support cross-parachain token transfers using [XCMP](https://github.com/paritytech/xcm-format/blob/master/README.md).

Since our parachain endeavours to be trustless, only reserve-backed transfers will be supported. Participating chains will need to agree on a parent chain to hold sovereign (reserve) accounts.

## Scenarios

### Alice transfers 21 PolkaETH to Bob on another chain

This transfer is modelled on [Transfer via reserve](https://github.com/paritytech/xcm-format/blob/master/README.md#transfer-via-reserve).

Parties:
- H: Home chain (Polkadot-Ethereum Bridge)
- D: Destination chain
- R: Reserve chain

Effects:
1. H will withdraw 21 PolkaETH from Alice's local account.
2. The sovereign account of H on R will be reduced by 21 PolkaETH.
3. The sovereign account of D on R will be credited with 21 PolkaETH.
4. D will mint 21 PolkaETH into Bob's account.

### Alice attempts to exchange 42 DOT for 21 PolkaETH

Alice holds 42 DOT on her home chain and wants to exchange it for 21 PolkaETH held on our parachain.

Parties:
- H: Home chain
- D: Destination chain (Polkadot-Ethereum Bridge)

H will need to have reserve holdings of DOT & PolkaETH on D.

Effects:
1. H will withdraw 42 DOT from Alice's local account.
2. D will perform the exchange, depositing 21 PolkaETH into Alice's local account.
3. H can take further action depending on the outcome of (2).

## Asset Identification

To support cross-parachain transfers, the consensus system needs to be able to identify the relative location of bridged ethereum assets held within our parachain.

These assets are all stored in our custom multi-asset [pallet](https://sad-curie-a48c3f.netlify.app/artemis_asset/index.html), and are individually identified by 20-byte identifiers. These identifiers will usually but not always correspond to a contract address on the Ethereum side.

Given this structure, the relative location for an asset can be determined using:
1. The index of our [asset](https://sad-curie-a48c3f.netlify.app/artemis_asset/index.html) pallet in the runtime.
2. The AccountId of the asset owner
3. The 20-byte asset identifier

This kind of path can modelled using various XCMP primitives:

```text
/ConcreteFungible/<parachain>/PalletInstance(<id>)/AccountId32/AccountKey20
```

## Outstanding Issues

### Numeric precision

Our parachain stores asset amounts using numbers with 256-bits of precision, while XCMP v0 only supports 128-bit numbers. We chose 256-bits because it matches Ethereum's numeric precision, and this prevents any chance of overflow occurring.

The simplest solution is keep our 256-bit precision, but perform checked conversion to 128-bits when required. This caps individual transfers to roughly 3.402 × 10<sup>29</sup> wei (3.402 × 10<sup>20</sup> eth), which is still a very huge amount.

The wider Substrate/Polkadot ecosystem seems to have settled on 128-bit precision, so it would be good to support that.
