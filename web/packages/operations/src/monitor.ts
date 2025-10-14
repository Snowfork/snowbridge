import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import { Context, environment, status, utils, subsquid, contextConfigFor } from "@snowbridge/api"
import { sendMetrics } from "./alarm"

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

    const context = new Context(contextConfigFor(env))

    const bridgeStatus = await status.bridgeStatusInfo(context, {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
    })
    console.log("Bridge Status:", bridgeStatus)

    let assethubChannelStatus = await status.channelStatusInfo(
        context,
        utils.paraIdToChannelId(config.ASSET_HUB_PARAID)
    )
    assethubChannelStatus.name = status.ChannelKind.AssetHub
    console.log("Asset Hub Channel:", assethubChannelStatus)

    const primaryGov = await status.channelStatusInfo(context, config.PRIMARY_GOVERNANCE_CHANNEL_ID)
    primaryGov.name = status.ChannelKind.Primary
    console.log("Primary Governance Channel:", primaryGov)

    const secondaryGov = await status.channelStatusInfo(
        context,
        config.SECONDARY_GOVERNANCE_CHANNEL_ID
    )
    secondaryGov.name = status.ChannelKind.Secondary
    console.log("Secondary Governance Channel:", secondaryGov)

    const [assetHub, bridgeHub, ethereum] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.ethereum(),
    ])

    let assetHubSovereign = BigInt(
        (
            (
                await bridgeHub.query.system.account(
                    utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID)
                )
            ).toPrimitive() as any
        ).data.free
    )
    console.log("Asset Hub Sovereign balance on bridgehub:", assetHubSovereign)

    let assetHubAgentBalance = await context
        .ethereum()
        .getBalance(
            await context
                .gateway()
                .agentOf(utils.paraIdToAgentId(bridgeHub.registry, config.ASSET_HUB_PARAID))
        )
    console.log("Asset Hub Agent balance:", assetHubAgentBalance)

    const bridgeHubAgentId = u8aToHex(blake2AsU8a("0x00", 256))
    let bridgeHubAgentBalance = await context
        .ethereum()
        .getBalance(await context.gateway().agentOf(bridgeHubAgentId))
    console.log("Bridge Hub Agent balance:", bridgeHubAgentBalance)

    console.log("Relayers:")
    let relayers = []
    for (const relayer of config.RELAYERS) {
        let balance = 0n
        switch (relayer.type) {
            case "ethereum":
                balance = await context.ethereum().getBalance(relayer.account)
                break
            case "substrate":
                balance = BigInt(
                    ((await bridgeHub.query.system.account(relayer.account)).toPrimitive() as any)
                        .data.free
                )
                break
        }
        relayer.balance = balance
        console.log("\t", balance, ":", relayer.type, "balance ->", relayer.name)
        relayers.push(relayer)
    }

    const channels = [assethubChannelStatus, primaryGov, secondaryGov]

    let sovereigns: status.Sovereign[] = [
        {
            name: "AssetHub",
            account: utils.paraIdToSovereignAccount("sibl", config.ASSET_HUB_PARAID),
            balance: assetHubSovereign,
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

    let indexerInfos: status.IndexerServiceStatusInfo[] = []
    const latestBlockOfAH = (await assetHub.query.system.number()).toPrimitive() as number
    const latestBlockOfBH = (await bridgeHub.query.system.number()).toPrimitive() as number
    const latestBlockOfEth = await ethereum.getBlockNumber()

    const chains = await subsquid.fetchLatestBlocksSynced(
        context.graphqlApiUrl(),
        env == "polkadot_mainnet"
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
            let latestBlockOnHydration = (await chain.query.system.number()).toPrimitive() as number
            let status = await subsquid.fetchSyncStatusOfParachain(context.graphqlApiUrl(), paraid)
            let info: status.IndexerServiceStatusInfo = {
                chain: status.name,
                paraid: status.paraid,
                latency: latestBlockOnHydration - status.height,
            }
            indexerInfos.push(info)
        }
    }
    console.log("Indexer service status:", indexerInfos)

    try {
        let latencies = await subsquid.fetchToEthereumUndelivedLatency(context.graphqlApiUrl())
        if (latencies && latencies.length) {
            assethubChannelStatus.toEthereum.undeliveredTimeout = latencies[0].elapse
        }
        latencies = await subsquid.fetchToPolkadotUndelivedLatency(context.graphqlApiUrl())
        if (latencies && latencies.length) {
            assethubChannelStatus.toPolkadot.undeliveredTimeout = latencies[0].elapse
        }
    } catch (error) {
        console.error("Failed to fetch undelivered latency:", error)
    }
    console.log("Asset Hub Channel with delivery timeout:", assethubChannelStatus)

    let v2Status

    try {
        v2Status = await status.v2Status(context)
        let latencies = await subsquid.fetchToEthereumV2UndelivedLatency(context.graphqlApiUrl())
        if (latencies && latencies.length) {
            v2Status.toEthereum.undeliveredTimeout = latencies[0].elapse
        }
        latencies = await subsquid.fetchToPolkadotV2UndelivedLatency(context.graphqlApiUrl())
        if (latencies && latencies.length) {
            v2Status.toPolkadot.undeliveredTimeout = latencies[0].elapse
        }
    } catch (error) {
        console.error("Failed to fetch undelivered latency:", error)
    }

    const allMetrics: status.AllMetrics = {
        name,
        bridgeStatus,
        channels,
        relayers,
        sovereigns,
        indexerStatus: indexerInfos,
        v2Status,
    }

    await sendMetrics(allMetrics)

    await context.destroyContext()

    return allMetrics
}
