import "dotenv/config"
import { createApi } from "@snowbridge/api"
import { ValidationKind } from "@snowbridge/api/dist/types/toPolkadot"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { Wallet } from "ethers"
import { bridgeInfoFor } from "@snowbridge/registry"

export const createAgent = async () => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const info = bridgeInfoFor(env)
    const api = createApi({ info, ethereumProvider: new EthersEthereumProvider() })
    const context = api.context

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC)

    console.log("Agent Creation on Snowbridge V2")
    {
        // Step 0. Create an agent implementation
        const creator = api.createAgent()
        const parachainAccount = "5CXiZE6z6w78EuqGdmJao7PFnmArgoHJbHbjWPftW5otnBKs"
        const agentId = await creator.agentIdForAccount(
            info.registry.assetHubParaId,
            parachainAccount,
        )

        console.log("Source parachain account:", parachainAccount)
        console.log("Creating agent with ID:", agentId)

        // Step 1. Create an agent creation tx
        const creation = await creator.tx(ETHEREUM_ACCOUNT_PUBLIC, agentId)

        // Step 2. Validate the transaction.
        const agentCreate = await creator.validate(creation)

        // Check validation logs for errors
        const errorLogs = agentCreate.logs.filter((l: any) => l.kind === ValidationKind.Error)
        if (errorLogs.length > 0) {
            console.error("Validation failed with errors:")
            errorLogs.forEach((log: any) => {
                console.error(`  [ERROR] ${log.message}`)
            })
            throw Error(`Validation has ${errorLogs.length} error(s).`)
        }

        console.log("agent create result", agentCreate)

        if (process.env["DRY_RUN"] != "true") {
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
                source account: ${parachainAccount}
                agent address: ${await context.gatewayV2().agentOf(agentId)}`)
        } else {
            console.log(
                `DRY_RUN mode: Agent would be created with ID ${agentId} for source account ${parachainAccount}`,
            )
        }
    }
    await context.destroyContext()
}

// Only run if this is the main module (not imported)
if (require.main === module) {
    createAgent()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
}
