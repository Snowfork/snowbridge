import { Keyring } from "@polkadot/keyring"
import { assetsV2, Context, contextConfigFor, historyV2, toEthereumV2 } from "@snowbridge/api"
import { formatUnits, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { assetRegistryFor } from "@snowbridge/registry"
import { setTimeout } from "timers/promises"
;(async () => {
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

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    // Select the token you want to send. In this case we use Ether. The registry contains the list of tokens.
    const TOKEN_CONTRACT = assetsV2.ETHER_TOKEN_ADDRESS

    // Select the destination parachain. In this case it is Asset Hub.
    const SOURCE_PARACHAIN = 1000

    console.log("Asset Hub to Ethereum")
    // Step 1. Get the delivery fee for the transaction
    const fee = await toEthereumV2.getDeliveryFee(
        context, // The context
        SOURCE_PARACHAIN, // Source parachain Id
        registry, // The asset registry
        TOKEN_CONTRACT // The token being transferred
    )

    // Step 2. Create a transfer tx
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

    // Step 3. Estimate the cost of the execution cost of the transaction
    console.log("call: ", transfer.tx.inner.toHex())
    console.log("utx: ", transfer.tx.toHex())
    const feePayment = (
        await transfer.tx.paymentInfo(POLKADOT_ACCOUNT, { withSignedTransaction: true })
    ).toPrimitive() as any
    console.log(
        `execution fee (${transfer.computed.sourceParachain.info.tokenSymbols}):`,
        formatUnits(feePayment.partialFee, transfer.computed.sourceParachain.info.tokenDecimals)
    )
    console.log(
        `delivery fee (${registry.parachains[registry.assetHubParaId].info.tokenSymbols}): `,
        formatUnits(fee.totalFeeInDot, transfer.computed.sourceParachain.info.tokenDecimals)
    )

    // Step 4. Validate the transaction.
    const validation = await toEthereumV2.validateTransfer(
        context, // The context
        transfer
    )
    console.log("validation result", validation)

    // Step 5. Check validation logs for errors
    if (validation.logs.find((l) => l.kind == toEthereumV2.ValidationKind.Error)) {
        throw Error(`validation has one of more errors.`)
    }

    // Step 6. Submit transaction and get receipt for tracking
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
    console.log(
        `Success message with message id: ${response.messageId}
                block number: ${response.blockNumber}
                tx hash: ${response.txHash}`
    )

    // Step 7. Poll for message completion
    while (true) {
        const status = await historyV2.toEthereumTransferById(
            context.graphqlApiUrl(), // GraphQL endpoint to query
            response.messageId
        )
        if (status !== undefined && status.status !== historyV2.TransferStatus.Pending) {
            console.dir(status, { depth: 100 })
            console.log("tx complete:", historyV2.TransferStatus[status.status])
            break
        }
        console.dir(status, { depth: 100 })
        console.log("waiting for tx to be completed...")
        await setTimeout(60_000) // Wait 60 seconds between requests
    }

    // Clean up all open connections
    await context.destroyContext()
})()
