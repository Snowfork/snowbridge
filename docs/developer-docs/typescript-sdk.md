---
description: A guide on using the Snowbridge TypeScript SDK for integration.
---

# Typescript SDK

## Packages

The following packages are used in the Snowbridge SDK:

* [**@snowbridge/api**](https://www.npmjs.com/package/@snowbridge/api) This is the **main entry point** for developers integrating with Snowbridge. It provides all core interfaces and helper functions to initiate, validate, and send cross-chain transactions. It abstracts over the complexities of constructing and handling XCM messages, Ethereum transactions, and relayer coordination.
  * Use the [`toPolkadotV2`](../../web/packages/api/src/toPolkadot_v2.ts) module for sending packages from Ethereum -> Polkadot.
  * Use the [`toEthereumV2`](../../web/packages/api/src/toEthereum_v2.ts) module for sending packages from Polkadot -> Ethereum.
* [**@snowbridge/registry**](https://www.npmjs.com/package/@snowbridge/registry) This package contains the **asset and parachain registry** used by Snowbridge. It defines the list of supported tokens, parachains, and associated metadata (like contract addresses and decimals). It ensures your transfers use valid combinations of assets and destinations.
* [**@snowbridge/contract-types**](https://www.npmjs.com/package/@snowbridge/contract-types) Contains **TypeScript typings and contract ABIs** for the Ethereum contracts Snowbridge interacts with. Use this package to interact with contracts directly, or to extend SDK functionality.
* [**@snowbridge/contracts**](https://www.npmjs.com/package/@snowbridge/contracts) Provides deployed contract addresses and metadata for Snowbridge smart contracts on supported networks. This is useful when you need to interact with Snowbridge's **Ethereum-side contracts** directly.
* [**@snowbridge/base-types**](https://www.npmjs.com/package/@snowbridge/base-types) Defines **common data types** used throughout the SDK, such as asset representations, transfer objects, parachain locations, and more. These types are shared between `@snowbridge/api` and the registry.

## Examples

The following examples show how to do an Ether transfer from Ethereum to Polkadot, and back. The full example code can be viewed by following the stated links.

### Ethereum to Polkadot

Full example: [send\_ether\_from\_eth\_to\_assethub.ts](../../web/packages/operations/src/examples/send_ether_from_eth_to_assethub.ts)

Uses the `@snowbridge/api` [`toPolkadotV2`](../../web/packages/api/src/toPolkadot_v2.ts) package to send the transaction.

#### Setup

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
```

#### Step 1: Get Delivery Fee

Use `getDeliveryFee()` to calculate how much the user must pay in order to deliver the message across chains. This includes relayer fees and any protocol-specific gas or weight costs. Displaying this to the user upfront ensures clarity and reduces failed transactions due to underpayment.

```javascript
// Select the token you want to send. In this case we use Ether. The registry 
// contains the list of tokens.
const TOKEN_CONTRACT = assetsV2.ETHER_TOKEN_ADDRESS
// Select the destination parachain. In this case it is Asset Hub.
const SOURCE_PARACHAIN = 1000
const fee = await toPolkadotV2.getDeliveryFee(
    context, // The context
    registry, // Asset registry
    TOKEN_CONTRACT, // The erc20 token contract address
    DESTINATION_PARACHAIN // Destination parachain
)
```

#### Step 2: Create Transfer

The `createTransfer()` function generates a transfer object, which includes source and destination accounts, the amount to send, the token being transferred and the precomputed delivery fee. This object contains all data necessary to execute the cross-chain transfer and is later used for validation and signing.

```javascript
const amount = 15_000_000_000_000n // 0.000015 ETH
const transfer = await toPolkadotV2.createTransfer(
    registry, // Asset registry
    ETHEREUM_ACCOUNT_PUBLIC, // Source account
    POLKADOT_ACCOUNT_PUBLIC, // Destination account
    TOKEN_CONTRACT, // The erc20 token contract address
    DESTINATION_PARACHAIN, // Destination parachain
    amount, // Transfer Amount
    fee // The delivery fee
)
```

#### Step 3: Validate Transfer

Although optional, `validateTransfer()` is strongly recommended. It performs local checks and dry-runs the transaction (when possible) to ensure:

* The sender has enough funds
* The asset is supported
* The constructed transaction will succeed on-chain

This step can save users from wasting gas or fees on transactions that would otherwise revert.

```javascript
const validation = await toPolkadotV2.validateTransfer(
    context, // The context
    transfer // The transfer tx
)

if (!validation.success) {
    console.error(validation.logs)
    throw Error(`validation has one of more errors.`)
}
```

#### Step 4: Send Transaction

Finally, the transaction is signed and submitted to the source chain. Use the `Wallet` instance to send the transaction.

```javascript
const response = await ETHEREUM_ACCOUNT.sendTransaction(transfer.tx)
const receipt = await response.wait(1)
if (!receipt) {
    throw Error(`Transaction ${response.hash} not included.`)
}
```

### Polkadot to Ethereum

Full example: [send\_ether\_from\_assethub\_to\_eth.ts](../../web/packages/operations/src/examples/send_ether_from_assethub_to_eth.ts)

Uses the `@snowbridge/api` [`toEthereumV2`](../../web/packages/api/src/toEthereum_v2.ts) module to send the transaction.

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

