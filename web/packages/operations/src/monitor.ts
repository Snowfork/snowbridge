import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import { Context, status, utils, subsquidV2 } from "@snowbridge/api"
import { sendMetrics } from "./alarm"
import { monitorParams } from "./monitorConfig"
import { Environment } from "../../base-types/dist"
import { bridgeInfoFor } from "@snowbridge/registry"

function contextConfigOverrides(input: Environment): Environment {
    let config = { ...input }
    let injectedEthChains: { [ethChainId: string]: string } = {}
    for (const ethChainIdKey of Object.keys(input.ethereumChains)) {
        if (
            process.env[`ETHEREUM_RPC_URL_${ethChainIdKey}`] ||
            process.env[`NEXT_PUBLIC_ETHEREUM_RPC_URL_${ethChainIdKey}`]
        ) {
            injectedEthChains[ethChainIdKey] =
                process.env[`ETHEREUM_RPC_URL_${ethChainIdKey}`] ||
                (process.env[`NEXT_PUBLIC_ETHEREUM_RPC_URL_${ethChainIdKey}`] as string)
            continue
        }
        injectedEthChains[ethChainIdKey] = input.ethereumChains[ethChainIdKey]
    }
    config.ethereumChains = injectedEthChains
    config.beaconApiUrl =
        process.env["BEACON_RPC_URL"] ||
        process.env["NEXT_PUBLIC_BEACON_RPC_URL"] ||
        input.beaconApiUrl

    let injectedParachains: { [paraId: string]: string } = {}
    for (const paraIdKey of Object.keys(input.parachains)) {
        if (
            process.env[`PARACHAIN_RPC_URL_${paraIdKey}`] ||
            process.env[`NEXT_PUBLIC_PARACHAIN_RPC_URL_${paraIdKey}`]
        ) {
            injectedParachains[paraIdKey] = (process.env[`PARACHAIN_RPC_URL_${paraIdKey}`] ||
                process.env[`NEXT_PUBLIC_PARACHAIN_RPC_URL_${paraIdKey}`]) as string
            continue
        }
        injectedParachains[paraIdKey] = input.parachains[paraIdKey]
    }
    config.parachains = injectedParachains
    config.relaychainUrl =
        process.env["RELAY_CHAIN_RPC_URL"] ||
        process.env["NEXT_PUBLIC_RELAY_CHAIN_RPC_URL"] ||
        input.relaychainUrl

    return config
}

export const monitor = async (): Promise<status.AllMetrics> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const { environment: snowbridgeEnv } = bridgeInfoFor(env)
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { name } = snowbridgeEnv

    const context = new Context(contextConfigOverrides(snowbridgeEnv))

    const bridgeStatus = await status.bridgeStatusInfo(context, {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
    })

    const channels = await fetchChannelStatus(context, snowbridgeEnv)

    const { relayers, sovereigns } = await fetchBalances(context, snowbridgeEnv)

    let indexerStatus: status.IndexerServiceStatusInfo[] = []
    try {
        indexerStatus = await fetchIndexerStatus(context, snowbridgeEnv)
    } catch (e) {
        console.error("Failed to fetch indexer status, continuing without it:", e)
    }

    const allMetrics: status.AllMetrics = {
        name,
        bridgeStatus,
        channels,
        relayers,
        sovereigns,
        indexerStatus,
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

const fetchChannelStatus = async (context: Context, env: Environment) => {
    let assethubChannelStatus = await status.channelStatusInfo(
        context,
        utils.paraIdToChannelId(env.assetHubParaId),
    )
    assethubChannelStatus.name = status.ChannelKind.AssetHub

    const primaryGov = await status.channelStatusInfo(
        context,
        monitorParams[env.name].PRIMARY_GOVERNANCE_CHANNEL_ID,
    )
    primaryGov.name = status.ChannelKind.Primary

    const secondaryGov = await status.channelStatusInfo(
        context,
        monitorParams[env.name].SECONDARY_GOVERNANCE_CHANNEL_ID,
    )
    secondaryGov.name = status.ChannelKind.Secondary

    return [assethubChannelStatus, primaryGov, secondaryGov]
}

const fetchBalances = async (context: Context, env: Environment) => {
    const [bridgeHub, ethereum] = await Promise.all([context.bridgeHub(), context.ethereum()])

    let relayers = []
    for (const relayer of monitorParams[env.name].RELAYERS) {
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
                    utils.paraIdToSovereignAccount("sibl", env.assetHubParaId),
                )
            ).toPrimitive() as any
        ).data.free,
    )

    let sovereigns: status.Sovereign[] = [
        {
            name: "AssetHub",
            account: utils.paraIdToSovereignAccount("sibl", env.assetHubParaId),
            balance: assetHubSovereignBalance,
            type: "substrate",
        },
    ]
    return { relayers, sovereigns }
}

export const fetchIndexerStatus = async (context: Context, env: Environment) => {
    let indexerInfos: status.IndexerServiceStatusInfo[] = []
    // Allow runtime override of monitored parachains without changing defaults.
    let monitorChains = monitorParams[env.name].TO_MONITOR_CHAINS
    if (monitorChains && monitorChains.length) {
        for (const chain of monitorChains) {
            try {
                let latestBlock = 0
                if (chain.type === "substrate") {
                    if (chain.id.toString().startsWith("kusama")) {
                        latestBlock = (
                            await (
                                await context.kusamaParachain(Number(chain.id.split("_")[1]))
                            ).query.system.number()
                        ).toPrimitive() as number
                    } else {
                        latestBlock = (
                            await (await context.parachain(Number(chain.id))).query.system.number()
                        ).toPrimitive() as number
                    }
                } else if (chain.type === "ethereum") {
                    latestBlock = await context.ethChain(Number(chain.id)).getBlockNumber()
                }
                const status = await subsquidV2.fetchLatestBlockFromIndexer(
                    context.graphqlApiUrl(),
                    chain.id.toString(),
                )
                const info: status.IndexerServiceStatusInfo = {
                    chain: status.name,
                    id: status.paraid,
                    latency: latestBlock - status.height,
                }
                indexerInfos.push(info)
            } catch (e) {
                console.error(`Failed to fetch indexer status for chain ${chain.id}:`, e)
            }
        }
    }
    return indexerInfos
}
