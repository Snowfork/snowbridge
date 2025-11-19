---
description: Guide to create an Agent on Ethereum.
---

# Create Agent

To transact from Polkadot to Ethereum, you need to create an agent on Ethereum. An agent is similar to a sovereign account on Polkadot.

### Step 1: Get Agent ID

Snowbridge has a runtime API on BridgeHub to determine the agent ID that you should register. Use `controlV2Api::agentId(location)` to determine your agent ID. You can copy the location input from the screenshot below, replacing your AccountId32 with your source account on Polkadot.

The resulting ID will be your agent ID, which you need to use in the next step.

**Please note:** This API is not on available on Polkadot BridgeHub yet, it will be added soon. It is available on BridgeHub Westend and Paseo.

<figure><img src="../../../.gitbook/assets/Screenshot 2025-11-03 at 15.24.50.png" alt=""><figcaption></figcaption></figure>

### Step 2: Create Agent

You have the option of creating an agent through the Snowbridge SDK, or by calling the contract directly.

#### SDK

The Snowbridge V2 SDK supports creating an Agent:

```typescript
// Step 1. Create an agent creation tx
const creation = await agentCreationImpl.createAgentCreation(
    {
        ethereum: context.ethereum(),
    },
    registry,
    ETHEREUM_ACCOUNT_PUBLIC,
    agentId,
)

// Step 2. Validate the transaction.
const validation = await agentCreationImpl.validateAgentCreation(
    {
        ethereum: context.ethereum(),
        gateway: context.gatewayV2(),
    },
    creation,
)

// Check validation logs for errors
const errorLogs = validation.logs.filter(
    (l) => l.kind === toEthereumSnowbridgeV2.ValidationKind.Error,
)
if (errorLogs.length > 0) {
    console.error("Validation failed with errors:")
    errorLogs.forEach((log) => {
        console.error(`  [ERROR] ${log.message}`)
    })
    throw Error(`Validation has ${errorLogs.length} error(s).`)
}

console.log("validation result", validation)

// Step 3. Submit the transaction
const response = await ETHEREUM_ACCOUNT.sendTransaction(creation.tx)
const receipt = await response.wait(1)
if (!receipt) {
    throw Error(`Transaction ${response.hash} not included.`)
}

if (receipt.status !== 1) {
    throw Error(`Transaction ${receipt.hash} failed with status ${receipt.status}`)
}

console.log(`Agent created successfully!
    tx hash: ${receipt.hash}
    agent address: ${await context.gatewayV2().agentOf(agentId)}`)

```

The full script is available at [https://github.com/Snowfork/snowbridge/blob/main/web/packages/operations/src/create\_agent.ts](../../../../web/packages/operations/src/create_agent.ts)

#### Call Contract

You can call the `v2_createAgent` method directly on the Snowbridge gateway contract: [https://etherscan.io/address/0x27ca963c279c93801941e1eb8799c23f407d68e7#writeProxyContract](https://etherscan.io/address/0x27ca963c279c93801941e1eb8799c23f407d68e7#writeProxyContract)

Enter the ID from step 1 and click `Write`:

<figure><img src="../../../.gitbook/assets/Screenshot 2025-11-05 at 13.42.20.png" alt=""><figcaption></figcaption></figure>
