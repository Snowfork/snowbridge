---
layout: default
title: XCM Interface for Cross-Chain Applications
parent: Building With Snowbridge
permalink: /building-with-snowbridge/xcm-for-state/
nav_order: 3
---

# XCM Interface for Cross-Chain Applications

Draft
{: .label .label-yellow }

## Bridge Messaging

Our bridge will support arbitrary messaging between Ethereum smart contracts and parachain apps. We'll need XCMP messages to facilitate this communication. As XCMP is still quite immature and untested, the below ideas are still WIP and likely to change but should provide a starting point for thinking through an initial implementation and early experiments.

XCMP messages are asynchronous and sent in a fire-and-forget manner, but they have delivery guarantees. Messages sent from Parachains to Ethereum will need to specify a target contract for delivery. Parachains will need to register to be notified of messages coming from Ethereum, and will be notified when relevant messages come through for them.

At the application layer, parachain apps and Ethereum smart contracts that interact are responsible for being aware of and trusting each other and for determining the payload and interface of their own messages. The bridge just facilitates transfer, verification and routing of these messages to the requested target application. Our ETHApp and ERC20App are examples of pairs of custom FRAME + solidity applications that trust each other and specify a shared interface - although they're implemented as pallets, one could imagine them working similarily as seperate parachains.

Once an application living on another parachain has been registered, the bridge can issue XCM messages:

```text
Xcm::Transact {
  origin: *,
  call: SomeAppOnAnotherParachain.dispatch(payload)
}
```

In turn, the application would need to notify the bridge in order to send a message to the Ethereum side:

```text
Xcm::Transact {
  origin: *
  call: Bridge.notify(app_id, payload)
}
```

## Draft Notes/Ideas

### Options for dynamic XCM in Ethereum -> Polkadot direction

---

Option A: Encode Destination Parachain and Parachain Pallet in RLP on Ethereum. Consuming Parachain needs to have Ethereum-xcm-adapter to be able to interpret the message and route it to the right pallet.

Ethereum-side app (eg: ETHApp):
-> emits plain Ethereum event(target_parachain_index, target_pallet_index, payload)

Parachain A:
accepts Ethereum event (target_parachain_index, target_pallet_index, payload)
-> converts to handle(target_pallet_index, payload)
-> xcm via transact to handle(target_pallet_index, payload) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, payload)
-> xcm-executes the transact, sends to xcm-adapter pallet
-> xcm-adapter pallet processes handle(target_pallet_index, payload)
-> statically maps target_pallet_index to another dispatchable Call (eg: mint on ETHApp)
-> calls dispatchable ETHApp.mint(payload)
-> ETHApp receives dispatchable, rlp decodes payload, and processes it

---

+: easiest implementation, cheaper on Ethereum encoding side as just rlp encoding, parachain developer can control/update routing easily, explicit coupling between pallet and smart contract
-: depends on parachain adding the xcm-adapter pallet, requires static mapping from source

---

Option B: Encode Destination Parachain and Parachain Pallet in SCALE on Ethereum. Consuming Parachain can accept xcm via transact directly and routing to the right pallet happens automatically via xcm-executor.

Ethereum ETHApp:
-> SCALE encode routing information, ie SCALE(target_parachain_index, target_pallet_index, payload)
-> emits Ethereum event(SCALE(target_parachain_index, target_pallet_index, payload))

Parachain A:
accepts Ethereum event (SCALE(target_parachain_index, target_pallet_index, payload))
-> xcm via transact(SCALE(target_parachain_index, target_pallet_index, payload)) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, payload)
-> xcm-executes the transact, sends to ETHApp pallet, mint function
-> xcm-execute calls dispatchable ETHApp.mint(payload)
-> ETHApp receives dispatchable, rlp decodes payload, and processes it

---

+: no static mapping/adapter pallet needed, any Ethereum contract can interface with parachain B pallets without permission, though can only interface with parachain B pallets that understand RLP.
-: need to implement (int,int,byte) full SCALE Call encoding on Ethereum, and pay cost to do encoding on Ethereum, implicit coupling between parachain and smart contract, so any updates to parachain or its pallets can break the Ethereum side

---

Option C: Encode Destination Parachain and Parachain Pallet AND payload data in SCALE on Ethereum. Consuming Parachain can accept xcm via transact directly and routing to the right pallet happens automatically via xcm-executor.

Ethereum ETHApp:
-> SCALE encode routing information, ie SCALE(target_parachain_index, target_pallet_index, amount, receiver)
-> emits Ethereum event(SCALE(target_parachain_index, target_pallet_index, amount, receiver))

Parachain A:
accepts Ethereum event (SCALE(target_parachain_index, target_pallet_index, amount, receiver))
-> xcm via transact(SCALE(target_parachain_index, target_pallet_index, amount, receiver)) on second Parachain

Parachain B:
receives transact with Call::handle(target_pallet_index, amount, receiver)
-> xcm-executes the transact, sends to ETHApp pallet, mint function
-> xcm-execute calls dispatchable ETHApp.mint(amount, receiver)
-> ETHApp receives dispatchable and processes it

---

+: no static mapping/adapter pallet needed, any Ethereum contract can interface with parachain B pallets whatsover without permission, even if those pallets know nothing about Ethereum or the smart contract. allows potentially completely permissionless cross-chain calls.
-: need to implement full SCALE Call encoding on Ethereum and basically full SCALE encoder for any data type on Etheruem, and possibly pay even higher cost to do encoding on Ethereum, completely implicit decoupling between parachain and smart contract, so any updates to parachain or its pallets can break the Ethereum side

### Options for dynamic XCM in Polkadot -> Ethereum direction

---

Option A: Registration/Whitelisting between Parachain+Pallet and Ethereum Smart Contract

+: coupling between pallet and smart contract is explicit. safety is explicit.
-: doesnt allow for dynamic sending, so limits application composability. requires permissiones composition.

---

Option B: Dynamic/Permissionless Sending

extend commitments pallet:
struct Message {
address: H160,
payload: Vec<u8>,
nonce: u64,
sourceParachain
}

XCM::Transact::Call::Commitments.add(address, payload_in_rlp)

add {
determine extrinsic origin and extract parachain identifier from it
add that to message
}

-> Ethereum
ETHApp is responsible for checking that origin is authorized

+: allows for dynamic sending, no permission required to do xcm to other pallets. makes x-chain composability as unconstrained as smart contract composability
-: no coupling between pallet and smart contract, so safety is left up to the application layer (similar model to Ethereum smart contracts)
