---
layout: default
title: Incentivized Bridge
nav_order: 8
permalink: /concepts/incentivized-bridge
parent: Concepts and Architecture
---

# Incentivized Bridge

Draft
{: .label .label-yellow }

In progress...

## Draft/Notes on our incentivized bridge channels

As discussed above, an ordered channel can only truly be secure and trusted if it can solve the fee bloating problem, which is challenging without a trusted pegged asset. this makes it challenging to use an ordered channel for the bootstrap channel. alternatively, we could use an unordered channel for the bootstrap channel. here are 2 possible solutions:
1 - unordered channel, no incentives, open - simple and usable for bootstrap
2 - ordered channel, permissioned just to ethapp and snowdot, fixed fee, minimum-enforced mint/burn amount that is significantly higher than fixed fee and accounts for wild currency exchange and gas price risk - solves fee bloating problem by ensuring that anyone attempting to bloat the channel would have to take a significant loss to do so. incentive-compatible so long as currency exchange + gas price fluctuation calculation remains within expected bounds. still incentive compatible even if not, though becomes a bit cheaper to attack.

todo: dynamic blockchain idea - source chain commitments have bids for each item. items are only flushed out of queue once processed on destination chain and confirmation message relayed back to source chain. relayer can process any messages in any order they like (or maybe must enforce fee-ordering or maybe some other mechanism like new eip?). relayer must relay confirmation msg back to source chain to claim their reward.

## Cryptoeconomic Analysis

The initial planned incentivized bridge aims to provide strict message ordering and guaranteed eventual delivery under certain cryptoeconomic assumptions. With strict message ordering, the order needs to be defined in our parachain message commitments, which means that whether or not a message is accepted into the bridge also needs to be defined in our parachain message commitments. A fee market based solution, whereby relayes can decide which messages to relay would not work for this, so we plan to go with a simpler fixed-fee-incentive for our first production bridge.

First, we define some terms we'll use in our model for a given message being relayed:
 - $$ gp_e $$ - This is the most recent Gas price that the parachain knows about when the user submits their message to our parachain to be relayed. This is the gas price the parachain expects the relayer to have to pay.
 - $$ gp_a $$ - This is the actual Gas price that a relayer ends up paying on their transaction to Ethereum when they relay the message.
 - $$ g_d $$ - This is the portion of the gas cost of the Ethereum transaction when relaying corresponding to just message delivery costs, excluding message dispatch costs. This cost is known and predictable, as the Solidity code path followed is the same for every message delivered, irrespective of the message destination or payload.
 - $$ g_i $$ - This is the portion of the gas cost of the Ethereum transaction when relaying corresponding to just message dispatch costs. This is unpredictable, as it depends on the target application, but if the target application is trusted and has a predictable gas cost, it could be predictable.
 - $$ g $$ - This is the total gas cost of the Ethereum transaction to the relayer. $$ g = g_d + g_i $$
 - $$ f $$ - This is the fixed fee incentive for the relayer in ETH.

For this analysis, we assume that the target application is known and trusted, such that $$ g_i $$ is fully predictable and so $$ g $$ is also fully predictable. In practice, $$ g_i $$ may not be predictable, but will be capped at a maximum amount.

The price a user is charged to put their message into the bridge is defined as:

$$
 gp_e * g + f
$$

This price is taken as revenue by the relayer.

The expenses a relayer will have to pay for relaying the message over to ethereum will be:

$$
gp_a * g
$$

The relayer profit, ie, revenue minus expenses is:

$$
gp_e * g + f - gp_a * g \\
=(gp_e - gp_a) * g + f
$$

In order for the bridge to guarantee delivery, it needs to ensure that relayers are profitably incentivized to relay their messages, ie, the following constraint must hold:

$$
(gp_e - gp_a) * g + f > 0
$$

We can see that in the event of a sudden gas price spike on Ethereum, this may not hold. We aim to choose a value for $$ f $$ that will preserve this constraint most of the time, ensuring ongoing guaranteed delivery of the bridge. Sine $$ g $$ is constant and predictable, we can define $$ f $$ in terms of g to simplify this, ie, $$ f = f_g * g $$ and so the constraint becomes:

$$

(gp_e - gp_a) * g + (f_g * g) > 0 \\
f_g > gp_e - gp_a

$$

With an expected maximum 1 minute delay in gas data, we should ensure $$ f_g $$ can cover the maximum 1-minute gas spike. We guess $$ f_g = 100 $$ should suffice for this for our first production rollout, though still need to do a formal analysis to confirm.