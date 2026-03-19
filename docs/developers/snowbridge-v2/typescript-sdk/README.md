---
description: A guide on using the SDK for integration.
---

# SDK

The V2 SDK is very similar to the V1 SDK. It adds parameters such as `customXcm` for executing XCM on AssetHub or destination parachains, and fee-selection options such as `feeAsset` or `feeTokenLocation` depending on the route.

## Packages

Snowbridge V2 uses the same packages as the Snowbridge V1 SDK. The current stable SDK release series starts at **v1.0.0**.

### Example Scripts

We have a wide range of scripts using the Snowbridge SDK at [https://github.com/Snowfork/snowbridge/tree/main/web/packages/operations/src](https://github.com/Snowfork/snowbridge/tree/main/web/packages/operations/src), as examples of how to use the SDK and the bridge.

### Guides

Here are guides to integrate with Snowbridge V2:

* [Token transfer Ethereum -> Polkadot](e2p.md)
* [Token transfer Polkadot -> Ethereum](e2p-1.md)
* [Transact on AssetHub & Parachain](transact-ah.md)
* [Transact on Ethereum & L2s](transact-ethereum.md)
* [SDK Cases](cases.md)
