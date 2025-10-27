---
description: A guide for parachain integraters.
---

# V2 Parachain Integration

### Snowbridge V2 Protocol

Snowbridge V2 is a generalized messaging protocol, that supports token transfers (ERC-20s and Polkadot native assets), as well as arbitratry contract execution in both directions.

Snowbridge V2 protocol improves upon V1 with better fee handling, batching of messages, unordered message delivery and the ability to execute contract/extrinsics via generalized message passing. It builds on the [requirements for Snowbridge V1](../snowbridge-v1/parachain-integration.md).

#### General Parachain Requirements

1. Must use at least Polkadot-SDK `stable2506`
2. Support XCMv5 and have `pallet-xcm` set up.
3. Support Ether as an asset. Your parachain must have a pallet that can store assets such as `orml-tokens` or `pallet-assets`.
4. Allow paying for execution with Ether.
5. XCM instruction `AliasOrigin` should be supported (check XCM weights are not set to MAX)

#### ERC20 Token Bridge  (ENA)&#x20;

1. Register the ERC20 token contract with the [Snowbridge Gateway.](../../rococo-testnet/rococo-sepolia-token-transfers.md#registering-tokens) This process will automatically set the ERC20 on Asset Hub as well.
2. Your parachain must support a pallet which can register assets such as `orml-tokens` or `pallet-xcm`. The ERC20 token must be registered with that pallet.
3. You can reach out to the Snowbridge team on Github to enable your token in our UI and SDK.

#### Polkadot Native Assets Token Bridge  (PNA)&#x20;

1. Your asset must first be registered on Asset Hub in the `ForeignAssets` Pallet.
2. Your asset must be able to `Teleport` to Asset Hub.
3. Once your asset can be sent successfully between Asset Hub and your chain you can then register the Asset on bridge hub via the `EthereumSystem` pallet `registerToken` extrinsic.
4. You can reach out to the Snowbridge team on github to enable your token in our UI and SDK.

#### Generalised Message Passing

In order to pass messages arbitrary message calls between Ethereum you require the following to be set up.

**Polkadot to Ethereum**

The origin that sends messages will need to an Agent created on Ethereum to act on that origins behalf on Ethereum. An agent is simply a contract on Ethereum that is associated with an origin on the Polkadot side. Only messages from that origin can dispatch messages to the Agent. [See more on agents.](../../architecture/components.md#agent)

1. Design your pallet or extrinsic carefully and choose the origin that will be used to dispatch to Ethereum.
2. Create an Agent with that origin on Bridge Hub.

**Ethereum to Polkadot**

All messages are routed through Asset Hub and your parachain will need to allow Asset Hub to alias origins from Ethereum.

1. Allow Asset Hub to alias origins. Example configuration: [People System Chain](https://github.com/polkadot-fellows/runtimes/blob/93d62ed/system-parachains/people/people-polkadot/src/xcm_config.rs#L230)
2. Allow system chains to alias accounts. Example configuration: [People System Chain](https://github.com/polkadot-fellows/runtimes/blob/93d62ed/system-parachains/people/people-polkadot/src/xcm_config.rs#L229)

### Using assets other than ETH or DOT for fees

Asset Hub contains a built-in DEX and since all Snowbridge messages are router through Asset Hub, there is a chance to swap DOT or ETH for any other fee asset.

1. The Asset must be registered on Asset Hub.
2. There must be a pool created with enough liquidity to make fee prices stable.
3. There must be monitoring of the pool in place to make sure its not drained of liquidity.
