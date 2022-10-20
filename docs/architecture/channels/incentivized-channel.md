# Incentivized Channel

The incentivized channel enforces delivery of messages in the order that they are sent. It also adds incentives which provide guaranteed delivery with strong properties based on simple economic assumptions without the need for any kind of Oracle-based solution.

A common problem with bridges is in handling fluctuating gas prices and exchange rates across chains and assets. Often oracle-based solutions are used to deal with this.&#x20;

However, with the [Basic Channel](basic-channel.md) activated, we now have access to cross-chain pegged assets, and so we can use those as part of our relaying incentive model. This means using [wETH](../apps/ether.md) and [wDOT](../apps/dot.md) to cover costs for relayers so that they are not impacted by changing exchange rates.&#x20;

In the Parachain→Ethereum direction, users pay a fixed fee in wrapped Ether ([wETH](../apps/ether.md)) to send a message to Ethereum. Relayers are rewarded Ether on the Ethereum side.

In the same way, for the Ethereum→Parachain direction, users pay a fixed fee in wrapped DOT ([wDOT](../apps/dot.md)) to send a message to Ethereum. Relayers are rewarded DOT on the Parachain side.

The fees are fixed, calculated offline, and can be only be updated by governance.

## Fee Calculation

### Polkadot->Ethereum

In this direction, messages are batched together in a bundle for delivery. Therefore the aggregate fees for the bundle would need to cover:

1. Transaction fee paid by relayer for delivering the bundle
2. Incentivization reward paid to message relayer

Since bundle sizes are dynamic, a fixed per-message fee could potentially result in two extremes:

1. If a bundle contains only a single message, then the aggregate fees may not be enough to incentivize relayers to deliver the bundle.
2. If a bundle contains a large number of messages, then the relayer makes a large profit at the expense of users.

Our solution for the initial launch is err on the side of overcompensating relayers to ensure message delivery. Some major [improvements](incentivized-channel.md#post-launch-improvements) are on our roadmap though.

### Ethereum->Polkadot

In this direction, messages are not batched, which simplifies fee calculation greatly. Given the larger blockspace expected on BridgeHub compared to Ethereum, fees should remain low and fairly constant over time. The fee would need to cover:

1. Transaction fee paid by relayer for delivering the message
2. Incentivization reward paid to relayer

## Increasing cost efficiency

To increase the cost-efficiency for Polkadot->Ethereum fees, we are planning several improvements post-launch.

Since the recent Ethereum [EIP-1559](https://www.blocknative.com/blog/eip-1559-fees) upgrade, the `baseFeePerGas` field is available in the block header and can be tracked by our Ethereum light client. This means an on-chain fee estimator can more accurately and dynamically derive an appropriate fee without overcharging as much.

Another improvement is to let users choose how long they would be willing to wait for their message to be included in a bundle. A user willing to wait 1 hour would pay less than a user who needs their message to be instantly transferred. Architecturally, this means the outbound channel would be divided into multiple _lanes_, where accumulated messages will be held for certain amount of time before being bundled_:_

* 1 minute&#x20;
* 10 minutes
* 1 hour

Beyond these improvements, the most cost-efficient incentivation approach is a fee market system where relayers can quote their delivery fees and influence the messages included in a bundle. Given the effort and complexities in building a decentralized and secure fee market, we'll only consider implementing this until well after launch.

## Protocol Objects

### Messages inbound from Ethereum

```rust
pub struct Envelope<T: Config> {
    /// The address of the outbound channel on Ethereum that forwarded this message.
    pub channel: H160,
    /// The application on Ethereum where the message originated from.
    pub source: H160,
    /// A nonce for enforcing replay protection and ordering.
    pub nonce: u64,
    /// Fee paid by user (wDOT)
    pub fee: BalanceOf<T>,
    /// declared weight of payload. Used to calculate transaction fee.
    pub weight: u64,
    /// The inner payload generated from the source application.
    pub payload: Vec<u8>,
}
```

### Messages inbound from Polkadot

```rust
struct MessageBundle {
    // ID of app pallet on parachain.
    uint8 sourceChannelID;
    // A nonce for enforcing replay protection and ordering.
    uint64 nonce;
    /// Fee paid by user (wETH)
    uint128 fee;
    // All messages submitted by account in this commitment period. 
    Message[] messages;  
}

struct Message {
    // each message has a unique id distinct from the bundle nonce  
    uint64 id;
    // target smart contract
    address target;
    // payload to dispatch to target
    bytes payload;
}
```

## Implementation

Pallets:

* [Inbound channel](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/incentivized-channel/src/inbound)
* [Outbound channel](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/incentivized-channel/src/outbound)

Solidity Contracts:

* [IncentivizedInboundChannel.sol](../../../ethereum/contracts/IncentivizedInboundChannel.sol)
* [IncentivizedOutboundChannel.sol](../../../ethereum/contracts/IncentivizedOutboundChannel.sol)
