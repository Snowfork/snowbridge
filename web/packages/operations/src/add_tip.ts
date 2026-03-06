import { Keyring } from "@polkadot/keyring"
import { EthersEthereumProvider, addTip, createApi } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { bridgeInfoFor } from "@snowbridge/registry"

export const addTipToMessage = async () => {
    await cryptoWaitReady()

    // Parse command line arguments
    const args = process.argv.slice(2)
    if (args.length < 4) {
        console.error(
            "Expected arguments: `direction (Inbound/Outbound), nonce, tipAsset (ETH/DOT), tipAmount`",
        )
        process.exit(1)
    }

    const direction = args[0] as "Inbound" | "Outbound"
    const messageNonce = BigInt(args[1])
    const tipAsset = args[2] as "ETH" | "DOT"
    const tipAmount = BigInt(args[3])

    if (direction !== "Inbound" && direction !== "Outbound") {
        throw new Error("Direction must be 'Inbound' or 'Outbound'")
    }

    if (tipAsset !== "ETH" && tipAsset !== "DOT") {
        throw new Error("Tip asset must be 'ETH' or 'DOT'")
    }

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const info = bridgeInfoFor(env)
    const { registry } = info
    const context = createApi({ info, ethereumProvider: new EthersEthereumProvider() }).context

    // Get user's Polkadot account
    const keyring = new Keyring({ type: "sr25519" })
    const userAccount = keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Alice")
    console.log("User account:", userAccount.address)

    const assetHub = await context.assetHub()

    const tipParams = {
        direction,
        nonce: messageNonce,
        tipAsset,
        tipAmount,
    }

    // Step 1: Estimate the extrinsic fee
    const estimatedFee = await addTip.getFee(assetHub, registry, tipParams, userAccount.address)
    console.log("Estimated extrinsic fee:", estimatedFee, " DOT")

    // Step 2: Dry run the transaction
    const dryRunResult = await addTip.dryRunAddTip(
        assetHub,
        registry,
        tipParams,
        userAccount.address,
    )

    if (!dryRunResult.success) {
        throw new Error(`Dry run failed: ${dryRunResult.errorMessage}`)
    }
    console.log("Dry run successful")

    // Step 3: Create the tip transaction
    const tipTx = await addTip.createAddTip(assetHub, registry, tipParams)

    // Step 4: Sign and send if not a dry run
    if (process.env.DRY_RUN !== "true") {
        const response = await addTip.signAndSend(assetHub, tipTx, userAccount, {
            withSignedTransaction: true,
        })
        if (!response) {
            throw Error(`Transaction ${response} not included.`)
        }
        console.log(
            `Tip added successfully!
            block hash: ${response.blockHash}
            tx hash: ${response.txHash}`,
        )
    }

    await context.destroyContext()
}

if (require.main === module) {
    addTipToMessage()
        .then(() => {
            console.log("\nDone")
            process.exit(0)
        })
        .catch((error) => {
            console.error("\nError:", error)
            process.exit(1)
        })
}
