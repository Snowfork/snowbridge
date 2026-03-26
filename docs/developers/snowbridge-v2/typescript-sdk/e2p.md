---
description: A guide on using the Snowbridge TypeScript SDK for integration.
---

# Token transfer Ethereum -> Polkadot

Uses the `@snowbridge/api`  [toPolkadotSnowbridgeV2](../../../../web/packages/api/src/toPolkadotSnowbridgeV2.ts) package to send the transaction. Please ensure you've completed [Setup Steps](setup-steps.md) before proceeding with this guide.

#### Pre-step: Create Transfer Impl

<pre class="language-typescript"><code class="lang-typescript">// Select the token you want to send. In this case we use Ether. The registry 
// contains the list of tokens.
const DESTINATION_PARACHAIN = 1000
const TOKEN_CONTRACT = assetsV2.ETHER_TOKEN_ADDRESS
<strong>const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
</strong>    DESTINATION_PARACHAIN,
    registry,
    TOKEN_CONTRACT
)
</code></pre>

#### Step 1: Get Delivery Fee

Use `getDeliveryFee()` to calculate how much the user must pay in order to deliver the message across chains. This includes relayer fees and any protocol-specific gas or weight costs. Displaying this to the user upfront ensures clarity and reduces failed transactions due to underpayment.

```javascript
let fee = await transferImpl.getDeliveryFee(
    context,
    registry,
    TOKEN_CONTRACT,
    DESTINATION_PARACHAIN,
    relayerFee
)
```

#### Step 2: Create Transfer

The `createTransfer()` function generates a transfer object, which includes source and destination accounts, the amount to send, the token being transferred and the precomputed delivery fee. This object contains all data necessary to execute the cross-chain transfer and is later used for validation and signing.

```javascript
const amount = 15_000_000_000_000n // 0.000015 ETH
const transfer = await transferImpl.createTransfer(
    {
        ethereum: context.ethereum(),
        assetHub: await context.assetHub(),
        destination:
            destParaId !== registry.assetHubParaId
                ? await context.parachain(destParaId)
                : undefined,
    },
    registry,
    DESTINATION_PARACHAIN,
    ETHEREUM_ACCOUNT_PUBLIC, // Source account
    POLKADOT_ACCOUNT_PUBLIC, // Destination account on AH
    TOKEN_CONTRACT,
    amount,
    fee // from step 1
)
```

#### Step 3: Validate Transfer

Although optional, `validateTransfer()` is strongly recommended. It performs local checks and dry-runs the transaction (when possible) to ensure:

* The sender has enough funds
* The asset is supported
* The constructed transaction will succeed on-chain

This step can save users from wasting gas or fees on transactions that would otherwise revert.

```javascript
const validation = await transferImpl.validateTransfer(
    {
        ethereum: context.ethereum(),
        gateway: context.gatewayV2(),
        bridgeHub: await context.bridgeHub(),
        assetHub: await context.assetHub()
    },
    transfer
)

// Step 4. Check validation logs for errors
if (validation.logs.find((l) => l.kind == toPolkadotV2.ValidationKind.Error)) {
    throw Error(`validation has one of more errors.`)
}
```

#### Step 4: Send Transaction

Finally, the transaction is signed and submitted to the source chain. Use the `Wallet` instance to send the transaction.

```javascript
const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
const receipt = await response.wait(1)
if (!receipt) {
    throw Error(`Transaction ${response.hash} not included.`)
}
// Check the message was sent
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

### Polkadot to Ethereum

Full example: [send\_ether\_from\_assethub\_to\_eth.ts](../../../../web/packages/operations/src/examples/send_ether_from_assethub_to_eth.ts)

Uses the `@snowbridge/api` [`toEthereumV2`](../../../../web/packages/api/src/toEthereum_v2.ts) module to send the transaction.

#### Setup

This step prepares all required state and dependencies for the transfer operation. This includes loading the asset registry, initializing the context, which sets up the connections to Ethereum and Substrate-based networks and loading the user wallets for both Ethereum and Substrate chains.

```typescript
// Initialize polkadot-js crypto
await cryptoWaitReady()
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

// Select the token you want to send. In this case we use Ether. The registry contains the list of tokens.
const TOKEN_CONTRACT = assetsV2.ETHER_TOKEN_ADDRESS
// Select the destination parachain. In this case it is Asset Hub.
const SOURCE_PARACHAIN = 1000
```

#### Step 1: Get Delivery Fee

Use `getDeliveryFee()` to calculate how much the user must pay in order to deliver the message across chains. This includes relayer fees and any protocol-specific gas or weight costs. Displaying this to the user upfront ensures clarity and reduces failed transactions due to underpayment.

```javascript
const fee = await toEthereumV2.getDeliveryFee(
    context, // The context
    SOURCE_PARACHAIN, // Source parachain Id
    registry, // The asset registry
    TOKEN_CONTRACT // The token being transferred
)
```

#### Step 2: Create Transfer

The `createTransfer()` function generates a transfer object, which includes source and destination accounts, the amount to send, the token being transferred and the precomputed delivery fee. This object contains all data necessary to execute the cross-chain transfer and is later used for validation and signing.

```javascript
const amount = 15_000_000_000_000n // 0.000015 ETH
const transfer = await toEthereumV2.createTransfer(
    { sourceParaId: SOURCE_PARACHAIN, context }, // The context and source parachain
    registry, // The asset registry
    POLKADOT_ACCOUNT_PUBLIC, // The source account
    ETHEREUM_ACCOUNT_PUBLIC, // The destination account
    TOKEN_CONTRACT, // The transfer token
    amount, // The transfer amount
    fee // The fee
)
```

#### Step 3: Validate Transfer

Although optional, `validateTransfer()` is strongly recommended. It performs local checks and dry-runs the transaction (when possible) to ensure:

* The sender has enough funds
* The asset is supported
* The constructed transaction will succeed on-chain

This step can save users from wasting gas or fees on transactions that would otherwise revert.

```javascript
const validation = await toEthereumV2.validateTransfer(
    context, // The context
    transfer
)

if (!validation.success) {
    console.error(validation.logs)
    throw Error(`validation has one of more errors.`)
}

```

#### Step 4: Send Transaction

Finally, the transaction is signed and submitted to the source chain. Use the SDK helper `signAndSend()` which manages construction, signing, and submission of the extrinsic.

```javascript
const response = await toEthereumV2.signAndSend(
    context, // The context
    transfer,
    POLKADOT_ACCOUNT,
    { withSignedTransaction: true }
)
if (!response) {
    throw Error(`Transaction ${response} not included.`)
}
if (!response.messageId) {
    throw Error(
        `Transaction ${response} did not have a message id. Did your transaction revert?`
    )
}
```

