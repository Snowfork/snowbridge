# Fees and Channels

Users pay bridge fees in the native token of the source chain. This means that on Ethereum, users pay in ETH to send messages across to Polkadot. In the opposite direction, users pay a fee in DOT to send messages to Ethereum.

These collected fees are paid in out rewards to off-chain message relayers. This incentivizes them to keep the bridge operating.

It is important to note that each parachain has its own logical messaging _channel_ to Ethereum. Fees and rewards are configured on a per-channel basis.

This allows greater flexibility in allowing parachains to subsidise the messaging activity of their users.

## Rebalancing

Fees are collected on the source network (Ethereum or Polkadot), and rewards are paid out on the destination network (Ethereum or Polkadot).

Depending on various factors, it may be necessary for channel owners to periodically _rebalance_ the accounts used for fees and rewards, so that the rewards account does not become empty.
