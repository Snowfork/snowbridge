import { Context, environment } from "@snowbridge/api"
import { AbstractProvider } from "ethers"

export const run = async (): Promise<void> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config, name, ethChainId } = snowbridgeEnv

    const parachains: { [paraId: string]: string } = {}
    parachains[config.BRIDGE_HUB_PARAID.toString()] =
        process.env["BRIDGE_HUB_URL"] ?? config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    parachains[config.ASSET_HUB_PARAID.toString()] =
        process.env["ASSET_HUB_URL"] ?? config.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

    const ethChains: { [ethChainId: string]: string | AbstractProvider } = {}
    Object.keys(config.ETHEREUM_CHAINS).forEach(
        (ethChainId) => (ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId])
    )
    if (process.env["EXECUTION_NODE_URL"]) {
        ethChains[ethChainId.toString()] = process.env["EXECUTION_NODE_URL"]
    }

    const ctx = new Context({
        environment: name,
        ethereum: {
            ethChainId,
            ethChains,
            beacon_url: process.env["BEACON_NODE_URL"] || config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            parachains: parachains,
            relaychain: process.env["RELAY_CHAIN_URL"] || config.RELAY_CHAIN_URL,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
        graphqlApiUrl: process.env["GRAPHQL_API_URL"] || config.GRAPHQL_API_URL,
    })
    const relaychain = await ctx.relaychain()
    await relaychain.isReady
    console.log("Connected to relaychain:", relaychain.runtimeChain.toString())
    const beefyClient = await ctx.beefyClient()
    const startBlock = process.env["FISHERMAN_START_BLOCK"]
        ? parseInt(process.env["FISHERMAN_START_BLOCK"])
        : 23423100
    const pastEvents = await beefyClient.queryFilter(
        beefyClient.filters.NewTicket(),
        startBlock,
        startBlock + 1000
    )
    for (let event of pastEvents) {
        console.log("Past NewTicket:", event.args.relayer, event.args.blockNumber)
        console.log("tx:", event.transactionHash)
        let tx = await ctx.ethereum().getTransaction(event.transactionHash)
        const parseTransaction = beefyClient.interface.parseTransaction({
            data: tx?.data || "",
        })
        const commitment = parseTransaction?.args[0]
        const beefyBlockNumber = commitment?.blockNumber
        const beefyMMRRoot = commitment?.payload[0].data
        console.log("Beefy Block Number:", beefyBlockNumber)
        console.log("Beefy MMR Root:", beefyMMRRoot)
        const beefyBlockHash = await relaychain.rpc.chain.getBlockHash(beefyBlockNumber)
        console.log("Beefy Block Hash:", beefyBlockHash.toHex())
        const canonicalMMRRoot = await relaychain.rpc.mmr.root(beefyBlockHash)
        console.log("Canonical MMR Root:", canonicalMMRRoot.toHex())
        if (canonicalMMRRoot.toHex() != beefyMMRRoot) {
            console.error("MMR Root mismatch!")
            //Todo: send alarms
        } else {
            console.log("MMR Root match.")
        }
    }
}
