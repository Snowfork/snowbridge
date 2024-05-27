import { contextFactory, destroyContext, environment, subscan, history } from "@snowbridge/api"

const monitor = async () => {
    const subscanKey = process.env.REACT_APP_SUBSCAN_KEY ?? ""

    let env = "rococo_sepolia"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config } = snwobridgeEnv
    if (!config.SUBSCAN_API) throw Error(`Environment ${env} does not support subscan.`)

    const context = await contextFactory({
        ethereum: {
            execution_url: config.ETHEREUM_WS_API(process.env.REACT_APP_ALCHEMY_KEY ?? ""),
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            url: {
                bridgeHub: config.BRIDGE_HUB_WS_URL,
                assetHub: config.ASSET_HUB_WS_URL,
                relaychain: config.RELAY_CHAIN_WS_URL,
                parachains: config.PARACHAINS,
            },
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const ethBlockTimeSeconds = 12
    const polkadotBlockTimeSeconds = 9
    const ethereumSearchPeriodBlocks = (60 * 60 * 24 * 7 * 2) / ethBlockTimeSeconds // 2 Weeks
    const polkadotSearchPeriodBlocks = (60 * 60 * 24 * 7 * 2) / polkadotBlockTimeSeconds // 2 Weeks

    const assetHubScan = subscan.createApi(config.SUBSCAN_API.ASSET_HUB_URL, subscanKey)
    const bridgeHubScan = subscan.createApi(config.SUBSCAN_API.BRIDGE_HUB_URL, subscanKey)
    const relaychainScan = subscan.createApi(config.SUBSCAN_API.RELAY_CHAIN_URL, subscanKey)

    const [ethNowBlock, assetHubNowBlock, bridgeHubNowBlock] = await Promise.all([
        (async () => {
            const ethNowBlock = await context.ethereum.api.getBlock("finalized")
            if (ethNowBlock == null) throw Error("Cannot fetch block")
            return ethNowBlock
        })(),
        (async () => {
            const nowBlockHash = await context.polkadot.api.assetHub.rpc.chain.getFinalizedHead()
            const nowBlock = await context.polkadot.api.assetHub.rpc.chain.getHeader(nowBlockHash)
            return nowBlock
        })(),
        (async () => {
            const nowBlockHash = await context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead()
            const nowBlock = await context.polkadot.api.bridgeHub.rpc.chain.getHeader(nowBlockHash)
            return nowBlock
        })(),
    ])
    const resultsToEthereum = await history.toEthereumHistory(
        context,
        assetHubScan,
        bridgeHubScan,
        relaychainScan,
        {
            assetHub: {
                fromBlock: assetHubNowBlock.number.toNumber() - polkadotSearchPeriodBlocks,
                toBlock: assetHubNowBlock.number.toNumber(),
            },
            bridgeHub: {
                fromBlock: bridgeHubNowBlock.number.toNumber() - polkadotSearchPeriodBlocks,
                toBlock: bridgeHubNowBlock.number.toNumber(),
            },
            ethereum: {
                fromBlock: ethNowBlock.number - ethereumSearchPeriodBlocks,
                toBlock: ethNowBlock.number,
            },
        }
    )
    console.log(JSON.stringify(resultsToEthereum, null, 2))

    const resultsToPolkadot = await history.toPolkadotHistory(
        context,
        assetHubScan,
        bridgeHubScan,
        {
            assetHub: {
                fromBlock: assetHubNowBlock.number.toNumber() - polkadotSearchPeriodBlocks,
                toBlock: assetHubNowBlock.number.toNumber(),
            },
            bridgeHub: {
                fromBlock: bridgeHubNowBlock.number.toNumber() - polkadotSearchPeriodBlocks,
                toBlock: bridgeHubNowBlock.number.toNumber(),
            },
            ethereum: {
                fromBlock: ethNowBlock.number - ethereumSearchPeriodBlocks,
                toBlock: ethNowBlock.number,
            },
        }
    )
    console.log(JSON.stringify(resultsToPolkadot, null, 2))

    await destroyContext(context)
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
