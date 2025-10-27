---
description: A guide for parachain integraters for Snowbridge V1.
---

# V1 Parachain Integration

### Snowbridge V1 Protocol

Snowbridge V1 is a token bridge which supports ERC20 assets and Polkadot assets.

#### General Parachain Requirements

1. An HRMP channel must be set up between Asset Hub and your parachain.
2. Your parachain must use at least Polkadot-SDK version `stable2409`
3. `pallet-xcm` with at least XCMv4 (XCMv5 preferable).
4. XCM `dryRun` runtime apis. [Asset Hub Parachain](https://github.com/polkadot-fellows/runtimes/blob/d6c5bd34c51ba7f670278f19d19e53e6db5a6b48/system-parachains/asset-hubs/asset-hub-polkadot/src/lib.rs#L1769)
5. XCM `feePayment` runtime apis. [Asset Hub Parachain](https://github.com/polkadot-fellows/runtimes/blob/d6c5bd34c51ba7f670278f19d19e53e6db5a6b48/system-parachains/asset-hubs/asset-hub-polkadot/src/lib.rs#L1741)
6. Your parachain must support a pallet which can register assets such as `orml-tokens` or `pallet-xcm`. DOT must be registered with that pallet.
7. Allow Asset Hub to be a reserve for DOT.
8. Accept DOT as payment for XCM execution.
9. Allow Asset Hub to be a reserve for bridged Assets. [Hydration Parachain](https://github.com/galacticcouncil/hydration-node/pull/784)

#### ERC20 Token Bridge  (ENA)&#x20;

1. Register the ERC20 token contract with the [Snowbridge Gateway.](../../rococo-testnet/rococo-sepolia-token-transfers.md#registering-tokens) This process will automatically set the ERC20 on Asset Hub as well.
2. Your parachain must support a pallet which can register assets such as `orml-tokens` or `pallet-xcm`. The ERC20 token must be registered with that pallet.
3. You can reach out to the Snowbridge team on github to enable your token in our UI and SDK.

#### Polkadot Native Assets Token Bridge  (PNA)&#x20;

1. Your asset must first be registered on Asset Hub in the `ForeignAssets` Pallet.
2. Your asset must be able to `Teleport` to Asset Hub.
3. Once your asset can be sent successfully between Asset Hub and your chain you can then register the Asset on bridge hub via the `EthereumSystem` pallet `registerToken` extrinsic.
4. You can reach out to the Snowbridge team on github to enable your token in our UI and SDK.
