import "dotenv/config"
import { EthersEthereumProvider, createApi, toPolkadotSnowbridgeV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { Wallet } from "ethers"
import { bridgeInfoFor } from "@snowbridge/registry"

export const registerTokenV2 = async (tokenAddress: string) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const info = bridgeInfoFor(env)
    const { registry } = info
    const context = createApi({ info, ethereumProvider: new EthersEthereumProvider() }).context

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC)

    const TOKEN_CONTRACT = tokenAddress.toLowerCase()

    console.log("Registering token:", TOKEN_CONTRACT)

    const relayerFee = 100_000_000_000_000n // 0.0001 ETH (~ $.5)

    console.log("Token Registration on Snowbridge V2")
    {
        // Step 0. Create a registration implementation
        const registrationImpl = toPolkadotSnowbridgeV2.createRegistrationImplementation()

        // Step 1. Get the registration fee for the transaction
        let fee = await registrationImpl.getRegistrationFee(context, registry, relayerFee)

        // Step 2. Create a registration tx
        const registration = await registrationImpl.createRegistration(
            context,
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            TOKEN_CONTRACT,
            fee,
        )

        // Step 3. Validate the transaction.
        const validation = await registrationImpl.validateRegistration(context, registration)

        // Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toPolkadotSnowbridgeV2.ValidationKind.Error)) {
            console.error("Validation failed with errors:")
            validation.logs.forEach((log) => {
                console.error(`  [${log.kind}] ${log.message}`)
            })
            throw Error(`validation has one or more errors.`)
        }

        console.log("validation result", validation)

        if (process.env["DRY_RUN"] != "true") {
            // Step 4. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(registration.tx)
            const receipt = await response.wait(1)
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            if (receipt.status !== 1) {
                throw Error(`Transaction ${receipt.hash} failed with status ${receipt.status}`)
            }

            // Step 6. Get the message receipt for tracking purposes
            const message = await toPolkadotSnowbridgeV2.getMessageReceipt(context, receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log(
                `Success message with nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`,
            )
        }
    }
    await context.destroyContext()
}

// Only run if this is the main module (not imported)
if (require.main === module) {
    if (process.argv.length != 3) {
        console.error("Expected arguments: `tokenAddress`")
        console.error("Example: npm run registerTokenV2 0x1234567890123456789012345678901234567890")
        process.exit(1)
    }

    registerTokenV2(process.argv[2])
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
}
