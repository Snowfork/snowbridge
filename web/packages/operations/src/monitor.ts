import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import { Context, environment, status, utils, subsquid } from "@snowbridge/api"
import { sendMetrics } from "./alarm"
import { BlockLatencyThreshold } from "./alarm"
import { AbstractProvider } from "ethers"

export const monitor = async (): Promise<status.AllMetrics> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config, name, ethChainId } = snowbridgeEnv

    const infuraKey = process.env.REACT_APP_INFURA_KEY || ""

    const parachains: { [paraId: string]: string } = {}
    parachains[config.BRIDGE_HUB_PARAID.toString()] =
        process.env["BRIDGE_HUB_URL"] ?? config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    parachains[config.ASSET_HUB_PARAID.toString()] =
        process.env["ASSET_HUB_URL"] ?? config.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

    const ethChains: { [ethChainId: string]: string | AbstractProvider } = {}
    Object.keys(config.ETHEREUM_CHAINS)
        .forEach(ethChainId => ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](infuraKey))
    if (process.env["EXECUTION_NODE_URL"]) { ethChains[ethChainId.toString()] = process.env["EXECUTION_NODE_URL"] }

    const context = new Context({
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

    const bridgeStatus = await status.bridgeStatusInfo(context, {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
        toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
        toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
    })
    console.log("Bridge Status:", bridgeStatus)

    const assethubChannelStatus = await status.channelStatusInfo(
        context,
        utils.paraIdToChannelId(config.ASSET_HUB_PARAID),
        {
            toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
            toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
        }
    )
    assethubChannelStatus.name = status.ChannelKind.AssetHub
    console.log("Asset Hub Channel:", assethubChannelStatus)

    const primaryGov = await status.channelStatusInfo(
        context,
        config.PRIMARY_GOVERNANCE_CHANNEL_ID,
        {
            toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
            toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
        }
    )
    primaryGov.name = status.ChannelKind.Primary
    console.log("Primary Governance Channel:", primaryGov)

    const secondaryGov = await status.channelStatusInfo(
        context,
        config.SECONDARY_GOVERNANCE_CHANNEL_ID,
        {
            toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
            toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
        }
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

    const chains = await subsquid.fetchLatestBlocksSynced()
    for (let chain of chains?.latestBlocks) {
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
    console.log("Indexer service status:", indexerInfos)

    const allMetrics: status.AllMetrics = {
        name,
        bridgeStatus,
        channels,
        relayers,
        sovereigns,
        indexerStatus: indexerInfos,
    }

    await sendMetrics(allMetrics)

    await context.destroyContext()

    return allMetrics
}
