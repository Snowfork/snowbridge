---
description: Guide to create an Agent on Ethereum.
---

# Create Agent

To transact from Polkadot to Ethereum, you need to create an agent on Ethereum. An agent is similar to a sovereign account on Polkadot.

### Step 1: Get Agent ID

Use the create-agent helper to derive the `agentId` for your source parachain account:

```typescript
import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { polkadot_mainnet } from "@snowbridge/registry"

const {
    chains: { assetHub },
} = polkadot_mainnet

const api = createApi({ info: polkadot_mainnet, ethereumProvider: new EthersEthereumProvider() })
const creator = api.createAgent()

const agentId = await creator.agentIdForAccount(
    assetHub.id,
    "5CXiZE6z6w78EuqGdmJao7PFnmArgoHJbHbjWPftW5otnBKs", // source account on the parachain
)
```

### Step 2: Create Agent

You have the option of creating an agent through the Snowbridge SDK, or by calling the contract directly.

#### SDK

The Snowbridge V2 SDK supports creating an Agent:

```typescript
const agentCreate = await creator.build(
    "0x...", // source Ethereum account submitting the create-agent transaction
    agentId,
)
```

The returned `agentCreate.tx` can then be submitted to the wallet by your application.

The full script is available at [https://github.com/Snowfork/snowbridge/blob/main/web/packages/operations/src/create\_agent.ts](../../../../web/packages/operations/src/create_agent.ts)

#### Call Contract

You can call the `v2_createAgent` method directly on the Snowbridge gateway contract: [https://etherscan.io/address/0x27ca963c279c93801941e1eb8799c23f407d68e7#writeProxyContract](https://etherscan.io/address/0x27ca963c279c93801941e1eb8799c23f407d68e7#writeProxyContract)

Enter the ID from step 1 and click `Write`:

<figure><img src="../../../.gitbook/assets/Screenshot 2025-11-05 at 13.42.20.png" alt=""><figcaption></figcaption></figure>
