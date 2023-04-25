# Incentivized Channel

_Note: This component is obsolete, and has been replaced with a new design. Docs will follow._&#x20;

The incentivized channel enforces delivery of messages in the order that they are sent. It also adds incentives which provide guaranteed delivery with strong properties based on simple economic assumptions without the need for any kind of Oracle-based solution.

A common problem with bridges is in handling fluctuating gas prices and exchange rates across chains and assets. Often oracle-based solutions are used to deal with this.&#x20;

However, with the [Basic Channel](basic-channel.md) activated, we now have access to cross-chain pegged assets, and so we can use those as part of our relaying incentive model. This means using [wETH](../apps/ether.md) and [wDOT](../apps/dot.md) to cover costs for relayers so that they are not impacted by changing exchange rates.&#x20;

In the Parachain→Ethereum direction, users pay a fixed upfront fee in wrapped Ether ([wETH](../apps/ether.md)) to send a message to Ethereum. Relayers are rewarded Ether on the Ethereum side.

In the same way, for the Ethereum→Parachain direction, users pay a fixed upfront fee in wrapped DOT ([wDOT](../apps/dot.md)) to send a message to Ethereum. Relayers are rewarded DOT on the Parachain side.

For our launch, the fees will be fixed, calculated offline, and can be only be updated by governance. Soon after launch, [dynamic fees](incentivized-channel.md#increasing-cost-efficiency) will be introduced.

## Pricing Strategy

### Polkadot->Ethereum

In this direction, messages are batched together in a bundle for delivery. Therefore the aggregate fees for the bundle would need to cover:

1. Transaction fee in Ether paid by relayer for delivering the bundle
2. Incentivization reward paid to message relayer

Since bundle sizes are dynamic, a fixed upfront fee could potentially result in two extremes:

1. If a bundle contains only a single message, then the aggregate fees may not be enough to incentivize relayers to deliver the bundle.
2. If a bundle contains a large number of messages, then the relayer makes a large profit at the expense of users.

Our solution for the initial launch is to charge an upfront unit fee that would cover the worst-case scenario: A message bundle containing a single message.

However, if finalized bundles end up containing more than one message, then participating users will be refunded by the amount that they overpaid.

### Ethereum->Polkadot

In this direction, messages are not batched, which simplifies pricing. Given the larger blockspace expected on BridgeHub compared to Ethereum, fees should remain low and fairly constant over time. The fee would need to cover:

1. Transaction fee in DOT paid by relayer for delivering the message
2. Incentivization reward paid to relayer

## Increasing cost efficiency

Since the recent Ethereum [EIP-1559](https://www.blocknative.com/blog/eip-1559-fees) upgrade, the `baseFeePerGas` field is available in the block header and can be tracked trustlessly by our Ethereum light client. This means an on-chain fee estimator can more accurately and dynamically derive an appropriate fee without overcharging as much.

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
