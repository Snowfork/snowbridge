---
description: A guide on using the Snowbridge TypeScript SDK for integration.
---

# Transact on AssetHub & Parachain

Uses the `@snowbridge/api` SDK.

#### Transact On AssetHub

To specify XCM to be executed on AssetHub, define your XCM:

```typescript
const remarkCall = moonbeam.tx.system.remarkWithEvent(remarkMessage)
const callHex = remarkCall.method.toHex()

// Get weight info for the call
const paymentInfo = await remarkCall.paymentInfo(POLKADOT_ACCOUNT_PUBLIC)
const weight = paymentInfo.weight

const customXcm = [
   {
       transact: {
           originKind: "SovereignAccount",
           fallbackMaxWeight: {
               refTime: weight.refTime.toBigInt(),
               proofSize: weight.proofSize.toBigInt(),
           },
           call: {
               encoded: callHex,
           },
       },
   },
]
```

This XCM uses the `system.remarkWithEvent` extrinsic, wrapped in a `Transact` XCM instruction.

#### Transact Execution

To execute the XCM program on AssetHub, the SDK integration is identical to the token transfer steps, with the extra `customXcm` parameter:

```typescript
import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { polkadot_mainnet } from "@snowbridge/registry"

const {
    chains: { ethereum, assetHub },
} = polkadot_mainnet
const api = createApi({ info: polkadot_mainnet, ethereumProvider: new EthersEthereumProvider() })

const sender = api.sender(ethereum, assetHub)

const transfer = await sender.build(
    "0x...", // source Ethereum account
    "5...", // beneficiary Polkadot account
    "0x0000000000000000000000000000000000000000", // Ether address
    15_000_000_000_000n, // amount: 0.000015 ETH
    {
        customXcm,
    },
)
```

The returned `transfer.tx` can then be submitted to the wallet by your application.

### Transact on Parachain

To transact on another parachain, like Hydration or NeuroWeb, use the same steps as above with concrete destructuring such as `const { chains: { ethereum, hydration } } = polkadot_mainnet`.

Your custom XCM program will be appended to the `InitiateTransfer` instruction that is built up in the SDK.

### Message Origin

It is important to note that the origin of the message on AssetHub or destination parachain is the original sender account on Ethereum, e.g.&#x20;

```
{
    parents: 2,
    interior: {
        x2: [
            {
                GlobalConsensus: {
                    Ethereum: {
                        chainId: 1,
                    },
                },
            },
            {
                AccountKey20: {
                    key: "0xa84670..."
                }
            }
        ],
    },
}
```

The destination parachain should support XCM instruction `AliasOrigin` , and the destination parachain should be able to map the origin location into an account, e.g. using `HashedDescription`.
