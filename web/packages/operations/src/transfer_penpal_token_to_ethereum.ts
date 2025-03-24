import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, environment, assetsV2, toEthereumV2 } from "@snowbridge/api"
import { formatUnits, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { readFile, writeFile } from "fs/promises"
import { existsSync } from "fs"

function cache<T>(filePath: string, generator: () => T | Promise<T>): Promise<T> {
    return (async () => {
        if (existsSync(filePath)) {
            // Read and parse existing cache file
            const data = await readFile(filePath)
            return JSON.parse(data.toString("utf-8"), (key, value) => {
                if (
                    typeof value === "string" &&
                    /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/.test(value)
                ) {
                    return new Date(value)
                }
                if (typeof value === "string" && /^bigint:\d+$/.test(value)) {
                    return BigInt(value.slice(7))
                }
                return value
            }) as T
        }

        // Generate new data and cache it
        const result = await generator()
        const json = JSON.stringify(
            result,
            (key, value) => {
                if (typeof value === "bigint") {
                    return `bigint:${value.toString()}`
                }
                return value
            },
            2
        )

        await writeFile(filePath, json)
        return result
    })()
}

const transfer = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { name, config, ethChainId } = snwobridgeEnv
    await cryptoWaitReady()

    const ethApikey = process.env.REACT_APP_INFURA_KEY || ""
    const ethChains: { [ethChainId: string]: string } = {}
    Object.keys(config.ETHEREUM_CHAINS).forEach(
        (ethChainId) =>
            (ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](ethApikey))
    )
    const context = new Context({
        environment: name,
        ethereum: {
            ethChainId,
            ethChains,
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            relaychain: config.RELAY_CHAIN_URL,
            parachains: config.PARACHAINS,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "your key goes here",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
        process.env.SUBSTRATE_KEY ?? "your key goes here"
    )
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)
    const amount = 1000000000n

    // Step 0. Build the Asset Registry. The registry contains the list of all token and parachain metadata in order to send tokens.
    // It may take some build but does not change often so it is safe to cache for 12 hours and shipped with your dapp as static data.
    //
    // The registry can be build from a snowbridge environment or snowbridge coutntext.
    //      const registry = await assetsV2.buildRegistry(assetsV2.fromEnvironment(snwobridgeEnv))
    // If your dapp does not use the snowbridge environment or context you can always build it manually by
    // specifying RegistryOptions for only the parachains you care about.

    const registry = await cache(
        `.${env}.registry.json`,
        async () => await assetsV2.buildRegistry(await assetsV2.fromContext(context))
    )
    console.log(
        "Asset Registry:",
        JSON.stringify(
            registry,
            (_, value) => (typeof value === "bigint" ? String(value) : value),
            2
        )
    )

    const assets = registry.ethereumChains[registry.ethChainId].assets

    const TOKEN_CONTRACT = Object.values(assets).find((t) =>
        t.name.toLowerCase().startsWith("pal")
    )!.token

    console.log("Penpal to Ethereum")
    {
        const sourceParaId = 1000
        // Step 1. Get the delivery fee for the transaction
        const fee = await toEthereumV2.getDeliveryFee(
            { assetHub: await context.assetHub(), source: await context.parachain(sourceParaId) },
            sourceParaId,
            registry,
            TOKEN_CONTRACT
        )

        // Step 2. Create a transfer tx
        const transfer = await toEthereumV2.createTransfer(
            await context.parachain(sourceParaId),
            registry,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            TOKEN_CONTRACT,
            amount,
            fee
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
        // console.log(
        //     "dryRun: ",
        //     (await transfer.tx.dryRun(POLKADOT_ACCOUNT, { withSignedTransaction: true })).toHuman()
        // )

        // Step 4. Validate the transaction.
        const validation = await toEthereumV2.validateTransfer(
            {
                sourceParachain: await context.parachain(sourceParaId),
                assetHub: await context.assetHub(),
                gateway: context.gateway(),
                bridgeHub: await context.bridgeHub(),
            },
            transfer
        )
        console.log("validation result", validation)

        // Step 5. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toEthereumV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 6. Submit transaction and get receipt for tracking
        const response = await toEthereumV2.signAndSend(
            await context.parachain(sourceParaId),
            transfer,
            POLKADOT_ACCOUNT,
            { withSignedTransaction: true }
        )
        if (!response) {
            throw Error(`Transaction ${response} not included.`)
        }
        console.log("Success message", response.messageId)
    }

    context.destroyContext()
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
