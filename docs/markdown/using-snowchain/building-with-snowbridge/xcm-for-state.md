---
layout: default
title: XCM Interface for Cross Chain Applications
parent: Building With Snowbridge
permalink: /building-with-snowbridge/xcm-for-state/
nav_order: 3
---

# XCM Interface for Cross-chain Applications

Draft
{: .label .label-yellow }

## Bridge Messaging

Our bridge will support arbitrary messaging between Ethereum smart contracts and parachain apps. We'll need XCMP messages to facilitate this communication. As XCMP is still quite immature and untested, the below ideas are still WIP and likely to change but should provide a starting point for thinking through an initial implementation and early experiments.

XCMP messages are asynchronous and sent in a fire-and-forget manner. Messages sent from Parachains to Ethereum will need to specify a target contract for delivery. Parachains will need to register to be notified of messages coming from Ethereum, and will be notified when relevant messages come through for them.

At the application layer, parachain apps and ethereum smart contracts that interact are responsible for being aware of and trusting each other and for determining the payload and interface of their own messages. The bridge just facilitates transfer, verification and routing of these messages to the requested target application. Our ETHApp and ERC20App are examples of pairs of custom FRAME + solidity applications that trust each other and specify a shared interface - although they're implemented as pallets, one could imagine them working similarily as seperate parachains.

Once an application living on another parachain has been registered, the bridge can issue XCM messages:

```text
Xcm::Transact {
  origin: *,
  call: SomeAppOnAnotherParachain.dispatch(payload)
}
```

In turn, the application would need to notify the bridge in order to unlock assets on the Ethereum side:

```text
Xcm::Transact {
  origin: *
  call: Bridge.notify(app_id, payload)
}
```
