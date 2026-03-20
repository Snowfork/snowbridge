---
description: A guide on using the Snowbridge TypeScript SDK for integration.
---

# Transact on Ethereum & L2s

Uses the `@snowbridge/api` SDK.

### Agent Setup

To execute arbitrary contracts on Ethereum and L2s, you need to create an agent for your calling parachain or user. More details can be found in the [Agents section](/broken/pages/gZd0UprOH4eSA5EYNy2H#agent).&#x20;

### SDK Usage

The SDK uses the same sender pattern as token transfers, with the Ethereum contract call passed through the build options.

```typescript
import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { polkadot_mainnet } from "@snowbridge/registry"

const {
    chains: { assetHub, ethereum },
} = polkadot_mainnet
const api = createApi({ info: polkadot_mainnet, ethereumProvider: new EthersEthereumProvider() })
const TARGET_CONTRACT = "0x1111111111111111111111111111111111111111"
const TARGET_CALLDATA = "0x"

const sender = api.sender(assetHub, ethereum)

const transfer = await sender.build(
    "5...", // source Polkadot account
    "0x...", // beneficiary Ethereum account
    "0x0000000000000000000000000000000000000000", // Ether address
    15_000_000_000_000n, // amount: 0.000015 ETH
    {
        fee: {
            contractCall: {
                target: TARGET_CONTRACT,
                calldata: TARGET_CALLDATA,
                value: 0n,
                gas: 500_000n,
            },
        },
    },
)
```

The returned `transfer.tx` can then be submitted to the wallet by your application.

For current route coverage, see [SDK Cases](cases.md).
