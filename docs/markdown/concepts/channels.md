---
layout: default
title: Channels
nav_order: 2
permalink: /concepts/channels
parent: Concepts and Architecture
---

# Channels

A channel is a concept used as part of the bridge which facilitates the delivery of multiple RPCs in a single direction. A channel consists of a sender and a receiver, each being a piece of business logic that runs on opposite chains. Any user or system wanting to send a message across the bridge must submit their RPC to the channel. Channels at the very least are used to provide some deliverability guarantees to a RPC message, and to provide replay protection across multiple messages.

## Channel Qualities

There are many different designs and ideas for how channels can be implemented, each with different tradeoffs and qualities. This page explores some of these and attempts to model out the domain of channel design such that we can eventually pick the design that suits our bridge best.

## Guaranteed Deliverability vs Guaranteed Delivery

As described [here](/concepts/components#deliverability-and-delivery), a channel must ensure deliverability, but it does not need to ensure delivery. Guranteed delivery is great for users and UX though, but complicated to get right and dependent on various assumptions. More complex channels can target both.

## Delivery vs Processing

Channels are responsible for delivering messages to their destination, but they do not have to be responsible for processing those messages. For example, a channel could deliver a message to Ethereum, but then expect applications/users to run a second transaction of their own to actually have those messages processed.

A channel that provided delivery and processing would likely lead to a simpler and easier UX though, so there is a desire to design channels that can provide both.

## Fees

Channels often involve fees. To deliver/process messages on the receiving chain, a channel will need to pay gas fees on the receiving chain. To bring in income to offset this payment, the channel will need to charge users fees too, or have an alternative compensation model.

### Receiver Delivery and Processing Gas Fees

Any transaction that runs through a channel will have to account for gas fees on the destination chain. Delivery fees can be predicted to some extent, as delivery will happen via the receiver code/logic which is controlled by the channel designer/developer, however processing fees may not be as predictable. If processing happens in the same transaction as delivery, fees could be unexpected and even unbounded. In these cases, channels must still be responsible for ensuring that they preserve deliverability even when fees may be unbounded. Preserving deliverability at the expense of successful processing is likely preferred, as the expectation for successful processing can be left up to the sending/receiving application to ensure.

### Sender Fees

When a user/application wants to submit to the channel, the channel will likely charge sender fees. At the very least, in order to preserve deliverability, a channel needs to ensure that it has enough fees to pay for gas for delivery on the receiving chain. It should design its sender fee/income model to aim to account for this.

One consideration is that the fee currency charged to senders is likely not the same as the gas fee currency paid on the receiving chain to do delivery/processing. Most likely the sending fee can only be charged in a currency that exists on the sending chain and the gas fee can only be paid in a currency that exists on the receiving chain. The channel then needs some calculation/process to ensure that it charges enough sender fees to cover the delivery fees it needs to pay. Essentially, calculating the delivery fee based on the currency of the sending fee is likely needed. An exchange rate, perhaps provided by an oracle, could be used for this calculation, but oracles come with their own problems. Incorrect fee/income management will risk draining the channel of its ability to pay for delivery, and so break the trust model.

If the same currency can be made available on both the sending and receiving chain, this may simplify some of the above issues. Snowbridge uses this approach by charging fees in SnowETH for the Polkadot to Ethereum direction. Ofcourse, SnowETH doesn't exist without a bridge, so there is a bootstrap problem, but we expect to be able to bootstrap SnowETH with a simple bridge design that only guarantees deliverability and then use it for a more powerful bridge design that guarantees delivery too.

Additionally, the delivery fee on Ethereum is charged based on gas, not Ether, with a gas price that fluctuates over time. For a Polkadot -> Ethereum channel, even if we can make Ether available on Polkadot to be used for channel sender fees, we still need to account for gas price volatility. The same concern applies on Parachains that have a market mechanism for fees, such as smart contract chains.

**Thought:** Perhaps there's a way for us to actually get Gas as a currency on Polkadot, like **PolkaGas**? _(actually, I think this idea for PolkaGas is an equivalent solution to some mechanisms that have been discussed whereby there are feedback/control messages from Ethereum to Polkadot via the bridge to update with the latest gasPrice to get an up-to-date exchange rate. I think building an isolated, simplified PolkaGas app would likely be a cleaner, simpler design than an integrated feedback/control message. It could also provide a PolkaGas/SnowETH exchange service based on the most recently known gas price)_

## Ordered vs Unordered

An ordered channel means that message are received and processed by the channel in the same order that they are sent into the channel. An unordered channel means that messages can be received and processed in any order.

In any channel that is shared by multiple users/applications, ordering will place additional constraints on those users/applications. In an ordered channel, every message becomes dependent on the successful acceptance and processing of all previous messages, meaning that users may become dependent on others to get their messages processed first. In order for a channel to preserve deliverability, this issue must be solved.

In an ordered channel, the cost of processing every message is dependent on cost of processing all past unprocessed messages too, and so it becomes challenging to bound the cost of a message. In the case of messages to Ethereum, with high gas costs, this needs to be dealt with. For example, a channel can set a max-processing-fee-per-message, so any messages higher fail. Message cost would still be unbounded though, as the potential number of past unprocessed messages could be unbounded. In practice though, an attacker trying to flood the channel would be bounded by their budget to spend on sender fees.

Bounds on the max throughput of the channel, limiting maximum messages over time could also be used as a stronger bound.

## Incentives

Besides channels charging sender fees to cover delivery/processing fees, they may also want to charge additional fees to cover incentives for third parties to actually participate in running the software to make the channel flow and relay messages.

The core value proposition for additional incentives is to ensure that a third part is incentivized to relay messages across the channel and pay destination fees so that users can get guarantees on delivery even when they are not capable of doing their own relaying, whether that's because they're offline or are being targeted for censorship or are just not able to run their own relaying software reliably.

Another value from incentivization is to solve the fee bloating problem of ordered channels. If the channel can be designed such that there is always a potential profit opportunity for relaying messages, then there are strong incentives for third party relayers or users with blocked messages to flush out bloated channels.

For both use cases above, incentivization should ideally ensure that the reward for relaying is higher than the cost for relaying, ie, sender fee is higher than delivery + processing fee.

As described above, incentive currencies may not be the same as fee currencies, so the same exchange rate calculation issues may apply. Consequently, there is the risk of a gas price spike that breaks the target of sender fee > delivery + processing fee and so breaks guaranteed delivery. The same solutions discussed above could be used here too.

## Permissioning

Channels could also be permissioned, i.e., restrict acceptance of messages or delivery of messages only to certain users/contracts/pallets/parachains.

They could also permission based on deeper payload inspection, only allowing for certain kinds of RPC payloads, essentially becoming more application-specific channels. An application-specific channel could remove some risk of uncertain/unbounded processing fees, as it would be able to predict processing fees based on being aware of the destination application code.

Such channels could come with their own qualities. E. g. an application could have a strictly ordered channel with low sender fees, where the fee for processing is charged by the receiver. If the receiver does not have enough balance, the message could be skipped.

Long term, we invision that there will be lower level general purpose channels with limited guarantees, a multitude of higher level channels to fit specific use cases, and specialized channels that optimize highly for specific applications.