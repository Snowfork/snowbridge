---
layout: default
title: XCMP Interface
parent: Specification
nav_order: 2
---

# XCMP Interface <!-- omit in toc -->

Draft
{: .label .label-yellow }

- [Introduction](#introduction)
- [Scenarios](#scenarios)
- [Asset Identification](#asset-identification)
- [Future Extensions](#future-extensions)
  - [Transfer from parachain to Ethereum](#transfer-from-parachain-to-ethereum)
  - [Bridge Messaging](#bridge-messaging)
- [Other Issues](#other-issues)
  - [Numeric precision](#numeric-precision)

## Introduction

Our parachain will support transfers of bridged assets using [XCMP](https://github.com/paritytech/xcm-format/blob/master/README.md).

Other parachains wanting to participate in asset transfers will need to hold sovereign reserves in our parachain. This implies a unilateral trust model where:

- Participating parachains must trust our parachain in its role as the reserve chain
- Our parachain does not need to trust the participants

## Asset Location

To support asset transfers we need to represent the relative location of bridged ethereum assets using the XCM [MultiAsset](https://github.com/paritytech/xcm-format/blob/master/README.md#multiasset-universal-asset-identifiers) structure.

Inside our parachain, the account balances for bridged ethereum assets are stored within our custom multi-asset [pallet](https://polkaeth-rustdocs.netlify.app/artemis_asset/index.html). Each asset is indentified by a 20-byte address that corresponds to a contract address on the Ethereum side. ETH is a special case and is identified by the zero address.

### ETH

```rust
MultiAsset::ConcreteFungible {
    id: MultiLocation::X2(
			Junction::PalletInstance { id: 11 },
      Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] })
    amount: AMOUNT,
  }
```

### ERC20

```rust
MultiAsset::ConcreteFungible {
    id: MultiLocation::X2(
			Junction::PalletInstance { id: 11 },
      Junction::AccountKey20 { network: NetworkId::Any, key: CONTRACT_ADDRESS })
    amount: AMOUNT,
  }
```

## Supported Transfers

We can implement support for the following scenarios using existing XCMP v0 message types.

### Alice transfers 21 PolkaETH to Bob on another chain <!-- omit in toc -->

This transfer is based on [Transfer via reserve](https://github.com/paritytech/xcm-format/blob/master/README.md#transfer-via-reserve), except that our parachain is acting as both the home chain and reserve chain.

Parties:

- H: Home chain (Ethereum Bridge)
- D: Destination chain

Effects:

1. H will withdraw 21 PolkaETH from Alice's local account.
2. The sovereign account of D on H will be credited with 21 PolkaETH.
3. D will mint 21 PolkaETH into Bob's account.

XCM Message:
```rust
Xcm::WithdrawAsset {
  assets: vec![ASSET],
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

### Alice transfers 21 PolkaETH to Bob on our chain <!-- omit in toc -->

Parties:

- H: Home chain
- D: Destination chain (Ethereum Bridge)

Effects:

1. H will withdraw 21 PolkaETH from Alice's local account.
2. The sovereign account of H on D will be reduced by 21 PolkaETH.
3. D will mint 21 PolkaETH into Bob's account.

XCM Message:

```rust
Xcm::WithdrawAsset {
  assets: vec![ASSET],
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

### Alice transfers 21 PolkaETH on chain X to Bob on chain Y <!-- omit in toc -->

In this scenario, our parachain is acting solely as the reserve chain for two other chains participating in a transfer.

Parties:

- H: Home chain
- D: Destination chain
- R: Reserve Chain (Ethereum Bridge)

Effects:

1. H will withdraw 21 PolkaETH from Alice's local account.
2. The sovereign account of H on R will be reduced by 21 PolkaETH.
3. The sovereign account of D on R will be credited with 21 PolkaETH.
4. D will mint 21 PolkaETH into Bob's account.

XCM Message:

```rust
Xcm::WithdrawAsset {
  assets: vec![ASSET],
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

We could also use XCMP to trigger a transfer of assets from our parachain to Ethereum, and vice versa. Since our parachain and our smart contracts on Ethereum have to trust each other, we could the [Teleportation](https://github.com/paritytech/xcm-format#transfer-via-teleport) mechanism described in the XCMP spec.

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

### Bridge Messaging

Our bridge will support arbitrary messaging between Ethereum smart contracts and parachain apps. We'll need XCMP messages to facilitate this communication. As XCMP is still quite immature and untested, the below ideas are still WIP and likely to change but should provide a starting point for thinking through an initial implementation and early experiments.

XCMP messages are asynchronous and sent in a fire-and-forget manner. Messages sent from Parachains to Ethereum will need to specify a target contract for delivery. Parachains will need to register to be notified of messages coming from Ethereum, and will be notified when relevant messages come through for them.

At the application layer, parachain apps and ethereum smart contracts that interact are responsible for being aware of and trusting each other and for determining the payload and interface of their own messages. The bridge just facilitates transfer, verification and routing of these messages to the requested target application. Our ETHApp and ERC20App are examples of pairs of substrate+solidity applications that trust each other and specify a shared interface - although they're implemented as pallets, one could imagine them working similarily as seperate parachains.

Example XCM sent to our parachain for sending a message to Ethereum:

```
SendMessageToContract {
  pallet_index: The index of the app module within the runtime
  contract_address: The address of the app's peer on the Ethereum side.
  payload: The payload of the message to be sent to the app's peer on the Ethereum side.
  effects: *
}
```

Example XCM sent to our parachain for registering to be notified about messages coming from Ethereum:
```
RegisterAppForNotification {
  pallet_index: The index of the app module within the runtime
  contract_address: The address of the app's peer on the Ethereum side.
  effects: *
}
```

Example XCM sent from our parachain to notify a listening parachain:
```
NewMessageFromEthereum {
  pallet_index: The index of the app module within the runtime
  payload: The payload of the message coming from the app's peer on the Ethereum side.
  nonce: ...
  contract_address: The address of the app's peer on the Ethereum side.
  effects: *
}
```

## Other Issues

### Numeric precision

Our parachain stores asset balances using 256-bit precision to match Ethereum, while XCMP v0 only supports 128-bit precision. Additionally most other asset pallets in the Polkadot ecosystem only support up to 128-bit precision.

The short term solution is keep our 256-bit precision, but perform checked conversion to 128-bits when required. This caps individual transfers to roughly 3.4 × 10<sup>38</sup> wei (3.4 × 10<sup>20</sup> eth), which is still a very huge amount.

In the longer term, for 256-bit precision to be supported in other parachains, we'll need to update [U256](https://docs.rs/primitive-types/0.7.2/primitive_types/struct.U256.html) so that its compatible with other asset pallet implementations.
