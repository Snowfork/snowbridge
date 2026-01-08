import "dotenv/config"
import { Context, toEthereumSnowbridgeV2, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"

// TODO add the ability to specify a location to create a agent from, using the EthereumSystemV2::agent_id API,
// once https://github.com/polkadot-fellows/runtimes/pull/978 has been released on-chain.
export const createAgent = async (agentId: string) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)

    console.log("Creating agent with ID:", agentId)

    console.log("Agent Creation on Snowbridge V2")
    {
        // Step 0. Create an agent creation implementation
        const agentCreationImpl = toEthereumSnowbridgeV2.createAgentCreationImplementation()

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
                agent address: ${await context.gatewayV2().agentOf(agentId)}`)
        } else {
            console.log(`DRY_RUN mode: Agent would be created with ID ${agentId}`)
        }
    }
    await context.destroyContext()
}

// Only run if this is the main module (not imported)
if (require.main === module) {
    if (process.argv.length != 3) {
        console.error("Expected arguments: `agentId`")
        console.error(
            "Example: npm run createAgent 0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314",
        )
        process.exit(1)
    }

    createAgent(process.argv[2])
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
}
