---
description: A guide on using the Snowbridge TypeScript SDK for Ethereum to Polkadot transfers.
---

# Token Transfer Ethereum -> Polkadot

Uses the `@snowbridge/api` SDK.

## Setup

```typescript
import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { polkadot_mainnet } from "@snowbridge/registry"

const {
    chains: { ethereum, assetHub },
} = polkadot_mainnet

const api = createApi({ info: polkadot_mainnet, ethereumProvider: new EthersEthereumProvider() })

const sender = api.sender(ethereum, assetHub)
```

## Build

```typescript
const transfer = await sender.build(
    "0x...", // source Ethereum account
    "5...", // beneficiary Polkadot account
    "0x0000000000000000000000000000000000000000", // Ether address
    15_000_000_000_000n, // amount: 0.000015 ETH
    {
        fee: {
            padFeeByPercentage: 33n,
        },
    },
)
```

The returned `transfer.tx` can then be submitted to the wallet by your application.
