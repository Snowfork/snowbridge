# Channels

Channels are the central organizing components of the bridge. Their main functions:

* Securely hauling messages between consensus domains
* Providing replay protection to ensure a message is not processed more than once
* Dispatching messages upon delivery

A full bidirectional channel consists of 4 related on-chain objects, a pair of _inbound_ and _outbound_ channels on both chains.

![](../../.gitbook/assets/1)

## Messaging Protocol

At the protocol layer, a message is embedded in an _envelope_ before being sent across the bridge. This envelope acts in much the same way as an IPv4 TCP or UDP packet, allowing a message payload to be routed correctly to its destination.

Inbound channels also verify the envelope of an incoming message to ensure it was emitted by its peer outbound channel on the Ethereum side.

## Message Dispatch

### Ethereum

Calls can be dispatched to any smart contract deployed on Ethereum. Each call has a maximum gas limit defined by the bridge. This prevents denial of service attacks by ensuring that all calls in message bundle can be executed in a single block.&#x20;

### Polkadot

Calls can be either be dispatched locally on the BridgeHub, by invoking a pallet's dispatchable, or forwarded to another parachain using an XCM message.

A call is dispatched locally using a custom origin. This origin identifies the smart contract which emitted the call into the outbound channel on Ethereum. Apps should verify the origin to implement  any necessary authentication.

Note:

* Message forwarding using XCM is still on our roadmap.

## Message Batching <a href="#_faw9foweutag" id="_faw9foweutag"></a>

Ethereum has much reduced blockspace than a typical Parachain, and gas costs are considerably higher.

As a mitigation for this, in the Parachain→Ethereum direction, messages are batched together into bundles and delivered to the Ethereum side. This reduces gas costs considerably.

The tradeoff is that these message bundles are limited in size, as all the messages in the bundle would need to be executed in a single Ethereum block. This means that the parachain can only accept a limited number of messages every batching period. This period is configurable, but for our launch we anticipate it being anywhere from 1 to 10 minutes.

## Implementations <a href="#_qd56myj60aib" id="_qd56myj60aib"></a>

Our bridge provides two concrete implementations, which differ primarily in their [delivery guarantees](../overview.md#deliverability-and-delivery).

* [Basic Channel](basic-channel.md)
* [Incentivized Channel](incentivized-channel.md)

Since the bridge has a modular architecture, new implementations can be added at any time.

