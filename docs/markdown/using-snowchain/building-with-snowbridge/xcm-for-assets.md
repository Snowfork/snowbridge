---
layout: default
title: XCM Interface for Assets
parent: Building With Snowbridge
permalink: /building-with-snowbridge/xcm-for-assets/
nav_order: 2
---

# XCM Interface for Assets

Draft
{: .label .label-yellow }

- [Introduction](#introduction)
- [Scenarios](#scenarios)
- [Asset Location](#asset-location)
- [Future Extensions](#future-extensions)
  - [Transfer from parachain to Ethereum](#transfer-from-parachain-to-ethereum)
  - [Bridge Messaging](#bridge-messaging)
- [Other Issues](#other-issues)
  - [Numeric precision](#numeric-precision)

## Introduction

Our parachain supports transfers of bridged assets using [XCMP](https://github.com/paritytech/xcm-format/blob/master/README.md).

Other parachains wanting to participate in asset transfers will need to hold sovereign reserves in our parachain. This implies a unilateral trust model where:

- Participating parachains must trust our parachain in its role as the reserve chain
- Our parachain does not need to trust the participants

Besides bridged assets, our parachain supports the transfer of DOT, the native currency of Polkadot.

## Asset Location

We can represent the relative location of bridged ethereum assets using the XCM [MultiAsset](https://github.com/paritytech/xcm-format/blob/master/README.md#multiasset-universal-asset-identifiers) structure.

Inside our parachain, the account balances for bridged ethereum assets are stored within our custom multi-asset [pallet](https://snowbridge-rust-docs.snowfork.com/artemis_asset/index.html). Each asset is indentified by a 20-byte address that corresponds to a contract address on the Ethereum side. ETH is a special case and is identified by the zero address.

Example: ETH

```rust
MultiAsset::ConcreteFungible {
  id: MultiLocation::X2(
    Junction::PalletInstance { id: 11 },
    Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] })
  amount: AMOUNT,
}
```

Example: ERC20

```rust
MultiAsset::ConcreteFungible {
  id: MultiLocation::X2(
    Junction::PalletInstance { id: 11 },
    Junction::AccountKey20 { network: NetworkId::Any, key: CONTRACT_ADDRESS })
  amount: AMOUNT,
}
```

## Supported Transfers

The following scenarios highlight the various kinds of asset transfers supported by our parachain.

### Transfer SnowETH from the reserve parachain to another parachain <!-- omit in toc -->

In this example, Alice wants to transfer 21 SnowETH to Bob on another parachain.

Parties:

- H: Home chain (Snowfork parachain)
- D: Destination chain

Effects:

1. H will withdraw 21 SnowETH from Alice's local account.
2. The sovereign account of D on H will be credited with 21 SnowETH.
3. D will mint 21 SnowETH into Bob's account.

Example Message:

```rust
Xcm::WithdrawAsset {
  assets: vec![MultiAsset::ConcreteFungible {
    id: MultiLocation::X2(
      Junction::PalletInstance { id: 11 },
      Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] })
    amount: AMOUNT,
  }],
  effects: vec![Order::DepositReserveAsset {
    assets: vec![MultiAsset::All],
    dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: DEST_PARA_ID }),
    effects: vec![Order::DepositAsset {
      assets: vec![MultiAsset::All],
      dest: MultiLocation::X1(Junction::AccountId32 {
        network: DEST_NETWORK,
        id: DEST_ACCOUNT,
      }),
    }],
  }],
}
```

### Transfer SnowETH from a parachain into the reserve parachain <!-- omit in toc -->

In this example, Alice transfers 21 SnowETH to Bob on the reserve chain

Parties:

- H: Home chain
- D: Destination chain (Snowfork parachain)

Effects:

1. H will withdraw 21 SnowETH from Alice's local account.
2. The sovereign account of H on D will be reduced by 21 SnowETH.
3. D will mint 21 SnowETH into Bob's account.

Example Message:

```rust
Xcm::WithdrawAsset {
  assets: vec![MultiAsset::ConcreteFungible {
    id: MultiLocation::X2(
      Junction::PalletInstance { id: 11 },
      Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] })
    amount: AMOUNT,
  }],
  effects: vec![Order::InitiateReserveWithdraw {
    assets: vec![MultiAsset::All],
    reserve: MultiLocation::X2(
      Junction::Parent,
      Junction::Parachain {
        id: DEST_PARA_ID,
      },
    ),
    effects: vec![Order::DepositAsset {
      assets: vec![MultiAsset::All],
      dest: MultiLocation::X1(Junction::AccountId32 {
          network: DEST_NETWORK,
          id: DEST_ACCOUNT,
      })
    }]
  }]
}
```

### Transfer SnowETH between 2 parachains via the reserve parachain <!-- omit in toc -->

In this scenario, our parachain is acting solely as the reserve chain for two other chains participating in a transfer.

Parties:

- H: Home chain
- D: Destination chain
- R: Reserve Chain (Snowfork parachain)

Effects:

1. H will withdraw 21 SnowETH from Alice's local account.
2. The sovereign account of H on R will be reduced by 21 SnowETH.
3. The sovereign account of D on R will be credited with 21 SnowETH.
4. D will mint 21 SnowETH into Bob's account.

Example Message:

```rust
Xcm::WithdrawAsset {
  assets: vec![MultiAsset::ConcreteFungible {
    id: MultiLocation::X2(
      Junction::PalletInstance { id: 11 },
      Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] })
    amount: AMOUNT,
  }],
  effects: vec![Order::InitiateReserveWithdraw {
    assets: vec![MultiAsset::All],
    reserve: MultiLocation::X2(
      Junction::Parent,
      Junction::Parachain {
          id: RESERVE_CHAIN,
      },
    ),
    effects: vec![Order::DepositReserveAsset {
      assets: vec![MultiAsset::All],
      dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: DEST_PARA }),
      effects: vec![Order::DepositAsset {
        assets: vec![MultiAsset::All],
        dest: MultiLocation::X1(Junction::AccountId32 {
          network: DEST_NETWORK,
          id: DEST_ACCOUNT,
        }),
      }],
    }]
  }]
}
```

## Future Extensions

These extensions are still being explored for feasibility and value.

### Transfer from parachain to Ethereum

We could also use XCMP to trigger a transfer of assets from our parachain to Ethereum, and vice versa. Since our parachain and our smart contracts on Ethereum have to trust each other, we could use the [Teleportation](https://github.com/paritytech/xcm-format#transfer-via-teleport) mechanism described in the XCMP spec.

We'll probably want to use a custom message type in the long-term though.

Example message sequence:

```
WithdrawAsset {
  assets: Assets to withdraw
  effects: [
    InitiateTeleport {
      assets: *
      dest: Destination ethereum network
      effects [
        DepositAsset {
          assets: *
          dest: AccountKey20
        }
      ]
    }
  ]
}
```

## Other Issues

### Numeric precision

Our parachain stores asset balances using 256-bit precision to match Ethereum, while XCMP v0 only supports 128-bit precision. Additionally most other asset pallets in the Polkadot ecosystem only support up to 128-bit precision.

The short term solution is keep our 256-bit precision, but perform checked conversion to 128-bits when required. This caps individual transfers to roughly 3.4 × 10<sup>38</sup> wei (3.4 × 10<sup>20</sup> eth), which is still a very huge amount.

In the longer term, for 256-bit precision to be supported in other parachains, we'll need to update [U256](https://docs.rs/primitive-types/0.7.2/primitive_types/struct.U256.html) so that its compatible with other asset pallet implementations.
