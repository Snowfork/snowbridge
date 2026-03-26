---
description: A guide on using the Snowbridge TypeScript SDK for integration.
---

# Transact on AssetHub & Parachain

Uses the `@snowbridge/api`  [toPolkadotSnowbridgeV2](../../../../web/packages/api/src/toPolkadotSnowbridgeV2.ts) module to send the transaction. Please ensure you've completed [Setup Steps](setup-steps.md) before proceeding with this guide.

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
const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
    ASSET_HUB_PARA_ID,
    registry,
    ETHER_TOKEN_ADDRESS // If you just want to send fees to cover the transact call
)

let fee = await transferImpl.getDeliveryFee(
    context,
    registry,
    ETHER_TOKEN_ADDRESS,
    ASSET_HUB_PARA_ID,
    relayerFee,
    {
        customXcm: customXcm, // specify custom XCM here
    }
)

const transfer = await transferImpl.createTransfer(
    context,
    registry,
    ASSET_HUB_PARA_ID,
    ETHEREUM_ACCOUNT_PUBLIC,
    POLKADOT_ACCOUNT_PUBLIC,
    ETHER_TOKEN_ADDRESS,
    amount,
    fee,
    customXcm // specify custom XCM here
)

const validation = await transferImpl.validateTransfer(context, transfer)

console.log("Validation result:")
validation.logs.forEach((log) => {
    console.log(`  [${log.kind}] ${log.message}`)
})
if (!validation.success) {
    throw Error("Validation failed")
}

const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
const receipt = await response.wait(1)
if (!receipt) {
    throw Error(`Transaction ${response.hash} not included.`)
}

const message = await toPolkadotSnowbridgeV2.getMessageReceipt(receipt)
if (!message) {
    throw Error(`Transaction ${receipt.hash} did not emit a message.`)
}
console.log(
    `Success message with nonce: ${message.nonce}
    block number: ${message.blockNumber}
    tx hash: ${message.txHash}`
)

```

### Transact on Parachain

To transact on another parachain, like Hydration or NeuroWeb, use the same steps as above, replacing the `ASSET_HUB_PARA_ID` constant with the parachain ID you would like to transact to.

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
