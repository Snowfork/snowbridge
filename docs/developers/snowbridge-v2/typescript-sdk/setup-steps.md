---
description: Snowbridge V2 setup steps.
---

# Setup Steps

Each Snowbridge integration requires certain environment variables to be set up. &#x20;

This step prepares all required state and dependencies for the transfer operation. This includes loading the asset registry, initializing the context, which sets up the connections to Ethereum and Substrate-based networks and loading the user wallets for both Ethereum and Substrate chains.

```typescript
// Initialize polkadot-js 
crypto await cryptoWaitReady()
// Get the registry of parachains and assets.
const environment = "polkadot_mainnet"
const registry = assetRegistryFor(environment)
// Initialize the context which establishes and pool connections
const context = new Context(contextConfigFor(environment))

// Initialize ethereum wallet.
const ETHEREUM_ACCOUNT = new Wallet(
    process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
    context.ethereum()
)
const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

// Initialize substrate wallet.
const polkadot_keyring = new Keyring({ type: "sr25519" })
const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
    process.env.SUBSTRATE_KEY ?? "Your Key Goes Here"
)
const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

// Set your relayer reward, in Ether. This should cost the extrinsic 
// fee to BridgeHub and XCM delivery fee to AssetHub.
const relayerFee = 100_000_000_000_000n 
```
