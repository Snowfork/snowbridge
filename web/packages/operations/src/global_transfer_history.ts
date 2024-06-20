import { contextFactory, destroyContext, environment, subscan, history } from "@snowbridge/api"
import { BeefyClient__factory, IGateway__factory } from "@snowbridge/contract-types"
import { AlchemyProvider } from "ethers"

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

    const { config, ethChainId } = snwobridgeEnv
    if (!config.SUBSCAN_API) throw Error(`Environment ${env} does not support subscan.`)

    const ethereumProvider = new AlchemyProvider(ethChainId, process.env.REACT_APP_ALCHEMY_KEY)
    const context = await contextFactory({
        ethereum: {
            execution_url: ethereumProvider,
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            url: {
                bridgeHub: config.BRIDGE_HUB_URL,
                assetHub: config.ASSET_HUB_URL,
                relaychain: config.RELAY_CHAIN_URL,
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
    const skipLightClientUpdates = true

    const [
        ethNowBlock,
        assetHubNowBlock,
        bridgeHubNowBlock,
        bridgeHubParaIdCodec,
        assetHubParaIdCodec,
    ] = await Promise.all([
        context.ethereum.api.getBlock("latest"),
        context.polkadot.api.assetHub.rpc.chain.getHeader(),
        context.polkadot.api.bridgeHub.rpc.chain.getHeader(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
    ])

    if (ethNowBlock == null) throw Error("Cannot fetch block")
    const bridgeHubParaId = bridgeHubParaIdCodec.toPrimitive() as number
    const assetHubParaId = assetHubParaIdCodec.toPrimitive() as number
    const beacon_url = context.config.ethereum.beacon_url
    const beefyClient = BeefyClient__factory.connect(config.BEEFY_CONTRACT, ethereumProvider)
    const gateway = IGateway__factory.connect(config.GATEWAY_CONTRACT, ethereumProvider)

    const [toEthereum, toPolkadot] = [
        await history.toEthereumHistory(
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
            },
            skipLightClientUpdates,
            ethChainId,
            assetHubParaId,
            beefyClient,
            gateway
        ),
        await history.toPolkadotHistory(
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
            },
            skipLightClientUpdates,
            bridgeHubParaId,
            gateway,
            ethereumProvider,
            beacon_url
        ),
    ]

    const transfers = [...toEthereum, ...toPolkadot]
    transfers.sort((a, b) => b.info.when.getTime() - a.info.when.getTime())
    console.log(JSON.stringify(transfers, null, 2))

    await destroyContext(context)
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
