import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"
import { ETHER_TOKEN_ADDRESS } from "@snowbridge/api/src/assets_v2"
import { DOT_LOCATION } from "@snowbridge/api/src/xcmBuilder"

const MOONBEAM_PARA_ID = 2004

export const transactToMoonbeam = async (amount: bigint, remarkMessage: string) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ??
            "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)
    const assetHub = await context.assetHub()
    const moonbeam = await context.parachain(MOONBEAM_PARA_ID)

    const TOKEN_CONTRACT = ETHER_TOKEN_ADDRESS
    const relayerFee = 100_000_000_000_000n // 0.0001 ETH

    console.log("Transacting to Moonbeam with custom XCM")
    {
        const remarkCall = moonbeam.tx.system.remarkWithEvent(remarkMessage)

        // Get weight info for the call
        const paymentInfo = await remarkCall.paymentInfo(POLKADOT_ACCOUNT_PUBLIC)
        const weight = paymentInfo.weight

        // Use .method.toHex() to get just the call data without extrinsic wrapper
        const callHex = remarkCall.method.toHex()

        console.log("Remark call weight:", weight.toString())
        console.log("Remark call hex (method only):", callHex)

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

        // Step 0. Create an ERC20ToParachain transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
            MOONBEAM_PARA_ID,
            registry,
            TOKEN_CONTRACT
        )

        // Step 1. Get the delivery fee for the transaction
        // Use DOT as fee asset since Moonbeam may not accept Ether for fees
        let fee = await transferImpl.getDeliveryFee(
            context,
            registry,
            TOKEN_CONTRACT,
            MOONBEAM_PARA_ID,
            relayerFee,
            {
                customXcm: customXcm,
                feeAsset: DOT_LOCATION,
            }
        )

        console.log("fee: ", fee)

        // Step 2. Create a transfer tx with custom XCM
        const transfer = await transferImpl.createTransfer(
            context,
            registry,
            MOONBEAM_PARA_ID,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            TOKEN_CONTRACT,
            amount,
            fee,
            customXcm
        )

        // Step 3. Validate the transaction
        console.log("Validating transaction...")
        const validation = await transferImpl.validateTransfer(context, transfer)

        console.log("Validation result:")
        validation.logs.forEach((log) => {
            console.log(`  [${log.kind}] ${log.message}`)
        })

        // Display dry run errors if present
        if (validation.data.assetHubDryRunError) {
            console.error("\nAsset Hub Dry Run Error:")
            console.error(validation.data.assetHubDryRunError)
        }
        if (validation.data.destinationParachainDryRunError) {
            console.error("\nMoonbeam Dry Run Error:")
            console.error(validation.data.destinationParachainDryRunError)
        }

        if (!validation.success) {
            console.error("\nValidation FAILED - transaction would likely fail")
            throw Error("Validation failed")
        }
        console.log("\nValidation SUCCESS - all checks passed")

        // Display transaction details
        console.log("\nTransaction Details:")
        console.log("  User ether balance:", formatEther(validation.data.etherBalance))
        if (validation.data.feeInfo) {
            console.log("  Estimated gas:", validation.data.feeInfo.estimatedGas.toString())
            console.log("  Gas price:", formatEther(validation.data.feeInfo.feeData.gasPrice ?? 0n), "ETH")
            console.log("  Execution cost:", formatEther(validation.data.feeInfo.executionFee))
            console.log("  Delivery cost:", formatEther(fee.totalFeeInWei))
            console.log("  Total TX cost:", formatEther(validation.data.feeInfo.totalTxCost))
        }
        console.log("  Ether sent to Moonbeam:", formatEther(amount))
        console.log("  Remark message:", remarkMessage)
        console.log("  Bridge status:", validation.data.bridgeStatus.toPolkadot.outbound)

        const { tx } = transfer

        if (process.env["DRY_RUN"] != "true") {
            console.log("sending tx")
            // Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            console.log("sent transaction")
            const receipt = await response.wait(1)
            console.log("got receipt")
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            // Get the message receipt for tracking purposes
            const message = await toPolkadotSnowbridgeV2.getMessageReceipt(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log(
                `Success message with nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`
            )
        }
    }
    await context.destroyContext()
}

if (process.argv.length != 4) {
    console.error("Expected arguments: `amount, remarkMessage`")
    console.error('Example: npm run transactToMoonbeam 100000000000000 "Hello from Ethereum!"')
    process.exit(1)
}

transactToMoonbeam(BigInt(process.argv[2]), process.argv[3])
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
