import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import {
    Context,
    environment,
    status,
    utils,
    subsquid,
    contextConfigFor,
    contextConfigOverrides,
} from "@snowbridge/api"
import { sendMetrics } from "./alarm"
import { Config } from "@snowbridge/api/dist/environment"

export const monitor = async (): Promise<status.AllMetrics> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config, name } = snowbridgeEnv

    const context = new Context(contextConfigOverrides(contextConfigFor(env)))

    const bridgeStatus = await status.bridgeStatusInfo(context, {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
    })

    const channels = await fetchChannelStatus(context, config)

    const { relayers, sovereigns } = await fetchBalances(context, config)

    let indexerStatus = await fetchIndexerStatus(context, env)

    let v2Status = await status.v2Status(context)

    const allMetrics: status.AllMetrics = {
        name,
        bridgeStatus,
        channels,
        relayers,
        sovereigns,
        indexerStatus,
        v2Status,
    }
    console.log(
        "All metrics:",
        JSON.stringify(
            allMetrics,
            (key, value) => {
                if (typeof value === "bigint") {
                    return `bigint:${value.toString()}`
                }
                return value
            },
            2,
        ),
    )

    await sendMetrics(allMetrics)

    await context.destroyContext()

    return allMetrics
}

const fetchChannelStatus = async (context: Context, config: Config) => {
    let assethubChannelStatus = await status.channelStatusInfo(
        context,
        utils.paraIdToChannelId(config.ASSET_HUB_PARAID),
    )
    assethubChannelStatus.name = status.ChannelKind.AssetHub

    const primaryGov = await status.channelStatusInfo(context, config.PRIMARY_GOVERNANCE_CHANNEL_ID)
    primaryGov.name = status.ChannelKind.Primary

    const secondaryGov = await status.channelStatusInfo(
        context,
        config.SECONDARY_GOVERNANCE_CHANNEL_ID,
    )
    secondaryGov.name = status.ChannelKind.Secondary

    return [assethubChannelStatus, primaryGov, secondaryGov]
}

const fetchBalances = async (context: Context, config: any) => {
    const [bridgeHub, ethereum] = await Promise.all([context.bridgeHub(), context.ethereum()])

    let relayers = []
    for (const relayer of config.RELAYERS) {
        let balance = 0n
        switch (relayer.type) {
            case "ethereum":
                balance = await ethereum.getBalance(relayer.account)
                break
            case "substrate":
                balance = BigInt(
                    ((await bridgeHub.query.system.account(relayer.account)).toPrimitive() as any)
                        .data.free,
                )
                break
        }
        relayer.balance = balance
        relayers.push(relayer)
    }

    let assetHubSovereignBalance = BigInt(
        (
            (
                await bridgeHub.query.system.account(
                    utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID),
                )
            ).toPrimitive() as any
        ).data.free,
    )

    let assetHubAgentBalance = await context
        .ethereum()
        .getBalance(
            await context
                .gateway()
                .agentOf(utils.paraIdToAgentId(bridgeHub.registry, config.ASSET_HUB_PARAID)),
        )

    const bridgeHubAgentId = u8aToHex(blake2AsU8a("0x00", 256))
    let bridgeHubAgentBalance = await context
        .ethereum()
        .getBalance(await context.gateway().agentOf(bridgeHubAgentId))

    let sovereigns: status.Sovereign[] = [
        {
            name: "AssetHub",
            account: utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID),
            balance: assetHubSovereignBalance,
            type: "substrate",
        },
        {
            name: "AssetHubAgent",
            account: utils.paraIdToAgentId(bridgeHub.registry, config.ASSET_HUB_PARAID),
            balance: assetHubAgentBalance,
            type: "ethereum",
        },
        {
            name: "BridgeHubAgent",
            account: u8aToHex(blake2AsU8a("0x00", 256)),
            balance: bridgeHubAgentBalance,
            type: "ethereum",
        },
    ]
    return { relayers, sovereigns }
}

export const fetchIndexerStatus = async (context: Context, env: string) => {
    const [assetHub, bridgeHub, ethereum] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.ethereum(),
    ])

    let indexerInfos: status.IndexerServiceStatusInfo[] = []
    const latestBlockOfAH = (await assetHub.query.system.number()).toPrimitive() as number
    const latestBlockOfBH = (await bridgeHub.query.system.number()).toPrimitive() as number
    const latestBlockOfEth = await ethereum.getBlockNumber()

    const chains = await subsquid.fetchLatestBlocksSynced(
        context.graphqlApiUrl(),
        env == "polkadot_mainnet",
    )
    for (let chain of chains) {
        let info: status.IndexerServiceStatusInfo = {
            chain: chain.name,
            latency: 0,
        }
        if (chain.name == "assethub") {
            info.latency = latestBlockOfAH - chain.height
        } else if (chain.name == "bridgehub") {
            info.latency = latestBlockOfBH - chain.height
        } else if (chain.name == "ethereum") {
            info.latency = latestBlockOfEth - chain.height
        }
        indexerInfos.push(info)
    }
    let monitorChains = context.monitorChains()
    if (monitorChains && monitorChains.length) {
        for (const paraid of monitorChains) {
            let chain = await context.parachain(paraid)
            let latestBlock = (await chain.query.system.number()).toPrimitive() as number
            let status = await subsquid.fetchSyncStatusOfParachain(context.graphqlApiUrl(), paraid)
            let info: status.IndexerServiceStatusInfo = {
                chain: status.name,
                paraid: status.paraid,
                latency: latestBlock - status.height,
            }
            indexerInfos.push(info)
        }
    }
    return indexerInfos
}
