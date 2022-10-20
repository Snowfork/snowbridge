# Overview

Our bridge has a layered architecture, inspired by networking protocols such as TCP/IP. At the lowest level we have channels, which send messages across the bridge. At the highest level, we have apps, which can invoke methods on apps living on foreign chains.

Our bridge is also general-purpose and modular by design. The goals for this include:

* Understandability: Getting to an easier to understand, simple design
* Flexibility: Having a design that supports seperation of incentives/guarantees with the ability to have multiple relaying designs with differing fees and guarantees
* Customizability: Having decoupling between pieces that allow different designs to depend on those pieces in whichever way they choose.

![High-level architecture](../.gitbook/assets/0)

## Definitions <a href="#_6hddi335yfdz" id="_6hddi335yfdz"></a>

* **Channel** - A bidirectional messaging channel responsible for accepting messages on the source chain and dispatching them on the target chain.&#x20;
* **Verifier** - An on-chain entity that verifies messages from a foreign chain. Internally it is implemented as a _light client_.
* **Light Client** - An on-chain client that is able to verify various aspects of blockchain state without having a full copy of all blocks and storage.
* **App** - An on-chain application that implements some cross-chain functionality, such as bridging ERC20 tokens or DOT. An app sends messages to an outbound channel, and receives messages from an inbound channel.
* **Relay** - An off-chain set of services that relays messages from outbound channels on the source chain to inbound channels on the destination chain.

## General-purpose <a href="#block-bbe1e16fb6614924a360297dcea763b2" id="block-bbe1e16fb6614924a360297dcea763b2"></a>

In the interoperability and bridge space, the default thing that comes to mind for most people and projects is the transfer of tokens from one blockchain to another. Most bridges tackle this piece of functionality first and design with it in mind.

However, interoperability is about more than just token transfers. Other kinds of assets, like non-fungible tokens, loan contracts, option/future contracts and generalized, type agnostic asset transfers across chains would be valuable functionality.

Being general-purpose, Snowbridge can facilitate transfer of arbitrary state across chains through arbitrary messages that are not tied to any particular application, and can support cross-chain applications.

## Deliverability and Delivery <a href="#deliverability-and-delivery" id="deliverability-and-delivery"></a>

In the context of this documentation, we often use the words guaranteed deliverability and guaranteed delivery. They both refer to different kinds of trust in the bridge.

If a bridge has _Guaranteed Deliverability_ it means that it is trustlessly possible for a message to be delivered across that bridge, ie, so long as someone is willing to run the software to relay the message and pay gas fees, it will be processed successfully and go through. _Guaranteed Deliverability_ does not mean that someone will actually do so - only that it is possible to do so without permission.

With _Guaranteed Deliverability_, the sender of the message can always deliver the message themself if they are willing to run a relayer and pay gas prices to do so, and so does not need to trust any third party if they don’t want to.

_Guaranteed Delivery_ on the other hand means that in addition, there are strong incentives or requirements for messages to be delivered such that based on economic assumptions, some third party will actually run software to relay messages and pay for gas and so messages will in fact be delivered even if the sender does not relay themself.
