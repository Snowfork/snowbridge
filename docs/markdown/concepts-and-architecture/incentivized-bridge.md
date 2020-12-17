---
layout: default
title: Incentivized Bridge
nav_order: 8
permalink: /concepts/incentivized-bridge
parent: Concepts and Architecture
---
In progress...

## Draft/Notes on our incentivized bridge channels
as discussed above, an ordered channel can only truly be secure and trusted if it can solve the fee bloating problem, which is challenging without a trusted peg asset. this makes it challenging to use an ordered channel for the bootstrap channel. alternatively, we could use an unordered channel for the bootstrap channel. here are 2 possible solutions:
 1 - unordered channel, no incentives, open - simple and usable for bootstrap
 2 - ordered channel, permissioned just to ethapp and snowdot, fixed fee, minimum-enforced mint/burn amount that is significantly higher than fixed fee and accounts for wild currency exchange and gas price risk - solves fee bloating problem by ensuring that anyone attempting to bloat the channel would have to take a significant loss to do so. incentive-compatible so long as currency exchange + gas price fluctuation calculation remains within expected bounds. still incentive compatible even if not, though becomes a bit cheaper to attack.

 todo: dynamic blockchain idea - source chain commitments have bids for each item. items are only flushed out of queue once processed on destination chain and confirmation message relayed back to source chain. relayer can process any messages in any order they like (or maybe must enforce fee-ordering or maybe some other mechanism like new eip?). relayer must relay confirmation msg back to source chain to claim their reward.