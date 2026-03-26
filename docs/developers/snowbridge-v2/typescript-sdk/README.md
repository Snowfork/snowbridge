---
description: A guide on using the SDK for integration.
---

# SDK

The V2 SDK is very similar to the V1 SDK. It adds new parameters such as `customXcm` (to execute XCMs on AssetHub or parachains) `feeLocation` (to specify which fee asset should be used).

## Packages

Snowbridge V2 uses the same packages as the Snowbridge V1 SDK. Snowbridge V2 methods are available from [https://www.npmjs.com/package/@snowbridge/api](https://www.npmjs.com/package/@snowbridge/api) > **v0.2.13**.

### Example Scripts

We have a wide range of scripts using the Snowbridge SDK at [https://github.com/Snowfork/snowbridge/tree/main/web/packages/operations/src](https://github.com/Snowfork/snowbridge/tree/main/web/packages/operations/src), as examples of how to use the SDK and the bridge.

### Guides

Here are guides to integrate with Snowbridge V2:

* [Token transfer Ethereum -> Polkadot](e2p.md)
* [Token transfer Polkadot -> Ethereum](e2p-1.md)
* [Transact on AssetHub & Parachain](transact-ah.md)
* [Transact on Ethereum & L2s](transact-ethereum.md)

