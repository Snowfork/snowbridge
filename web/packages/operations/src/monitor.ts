import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import { Context, status, utils, subsquidV2 } from "@snowbridge/api"
import { sendMetrics } from "./alarm"
import { Environment } from "../../base-types/dist"
import { bridgeInfoFor } from "@snowbridge/registry"

export const monitorParams: {
    [id: string]: {
        PRIMARY_GOVERNANCE_CHANNEL_ID: string
        SECONDARY_GOVERNANCE_CHANNEL_ID: string
        RELAYERS: {
            name: string
            account: string
            type: "substrate" | "ethereum"
            balance?: bigint
        }[]
        TO_MONITOR_PARACHAINS?: number[]
    }
} = {
    local_e2e: {
        PRIMARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        SECONDARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        RELAYERS: [
            {
                name: "beacon",
                account: "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c",
                type: "substrate",
            },
            {
                name: "beefy",
                account: "0x87D987206180B8f3807Dd90455606eEa85cdB87a",
                type: "ethereum",
            },
            {
                name: "parachain-primary-gov",
                account: "0xeEBFA6B9242A19f91a0463291A937a20e3355681",
                type: "ethereum",
            },
            {
                name: "parachain-secondary-gov",
                account: "0x13e16C4e5787f878f98a610EB321170512b134D4",
                type: "ethereum",
            },
            {
                name: "execution-assethub",
                account: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV",
                type: "substrate",
            },
            {
                name: "parachain-assethub",
                account: "0x8b66D5499F52D6F1857084A61743dFCB9a712859",
                type: "ethereum",
            },
            {
                name: "execution-penpal",
                account: "5HgmfVcc8xBUcReNJsUaJMhFWGkdYpEw4RiCX4SeZPdKXR6H",
                type: "substrate",
            },
            {
                name: "parachain-penpal",
                account: "0x01F6749035e02205768f97e6f1d394Fb6769EC20",
                type: "ethereum",
            },
        ],
    },
    paseo_sepolia: {
        PRIMARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        SECONDARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        RELAYERS: [
            {
                name: "beacon",
                account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                type: "substrate",
            },
            {
                name: "beefy",
                account: "0xc189De708158e75E5C88C0ABfA5F9a26C71F54D1",
                type: "ethereum",
            },
            {
                name: "parachain-primary-gov",
                account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                type: "ethereum",
            },
            {
                name: "parachain-secondary-gov",
                account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                type: "ethereum",
            },
            {
                name: "execution-assethub",
                account: "5HT2ysqEg6SXghQ3NGXp1VWT22hhj48Um8UAwk6Udg8ZCEv8",
                type: "substrate",
            },
            {
                name: "parachain-assethub",
                account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                type: "ethereum",
            },
        ],
    },
    polkadot_mainnet: {
        PRIMARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        SECONDARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        RELAYERS: [
            {
                name: "beacon",
                account: "16DWunYRv2q29SMxqgrPGhob5az332hhLggSj2Rysk3g1rvk",
                type: "substrate",
            },
            {
                name: "beefy",
                account: "0xB8124B07467E46dE73eb5c73a7b1E03863F18062",
                type: "ethereum",
            },
            {
                name: "beefy-on-demand",
                account: "0xF3D021D51a725F5DBDCE253248E826A8644Be3c1",
                type: "ethereum",
            },
            {
                name: "parachain-primary-gov",
                account: "0x0f51678Ac675C1abf2BeC1DAC9cA701cFcfFF5E2",
                type: "ethereum",
            },
            {
                name: "parachain-secondary-gov",
                account: "0x0f51678Ac675C1abf2BeC1DAC9cA701cFcfFF5E2",
                type: "ethereum",
            },
            {
                name: "execution-assethub",
                account: "13Dbqvh6nLCRckyfsBr8wEJzxbi34KELwdYQFKKchN4NedGh",
                type: "substrate",
            },
            {
                name: "parachain-assethub",
                account: "0x1F1819C3C68F9533adbB8E51C8E8428a818D693E",
                type: "ethereum",
            },
        ],
        TO_MONITOR_PARACHAINS: [2034, 2043, 3369], // Hydration, OriginTrail, Mythos
    },
    westend_sepolia: {
        PRIMARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        SECONDARY_GOVERNANCE_CHANNEL_ID:
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        RELAYERS: [
            {
                name: "beacon",
                account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                type: "substrate",
            },
            {
                name: "beefy",
                account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                type: "ethereum",
            },
            {
                name: "parachain-primary-gov",
                account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                type: "ethereum",
            },
            {
                name: "parachain-secondary-gov",
                account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                type: "ethereum",
            },
            {
                name: "execution-assethub",
                account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                type: "substrate",
            },
            {
                name: "parachain-assethub",
                account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                type: "ethereum",
            },
        ],
    },
}

const parseMonitorParachainsOverride = (): number[] | undefined => {
    const raw = process.env["MONITOR_PARACHAINS"]
    if (!raw) {
        return undefined
    }
    const parsed = raw
        .split(/[\s,]+/)
        .map((value) => Number.parseInt(value, 10))
        .filter((value) => Number.isFinite(value))
    return parsed.length ? parsed : undefined
}

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

    let indexerStatus = await fetchIndexerStatus(context, snowbridgeEnv)

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

    let assetHubAgentBalance = await context
        .ethereum()
        .getBalance(
            await context
                .gateway()
                .agentOf(utils.paraIdToAgentId(bridgeHub.registry, env.assetHubParaId)),
        )

    const bridgeHubAgentId = u8aToHex(blake2AsU8a("0x00", 256))
    let bridgeHubAgentBalance = await context
        .ethereum()
        .getBalance(await context.gateway().agentOf(bridgeHubAgentId))

    let sovereigns: status.Sovereign[] = [
        {
            name: "AssetHub",
            account: utils.paraIdToSovereignAccount("sibl", env.assetHubParaId),
            balance: assetHubSovereignBalance,
            type: "substrate",
        },
        {
            name: "AssetHubAgent",
            account: utils.paraIdToAgentId(bridgeHub.registry, env.assetHubParaId),
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

export const fetchIndexerStatus = async (context: Context, env: Environment) => {
    const [assetHub, bridgeHub, ethereum] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.ethereum(),
    ])

    let indexerInfos: status.IndexerServiceStatusInfo[] = []
    const latestBlockOfAH = (await assetHub.query.system.number()).toPrimitive() as number
    const latestBlockOfBH = (await bridgeHub.query.system.number()).toPrimitive() as number
    const latestBlockOfEth = await ethereum.getBlockNumber()

    const chains = await subsquidV2.fetchLatestBlocksSynced(
        context.graphqlApiUrl(),
        env.name == "polkadot_mainnet",
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
    // Allow runtime override of monitored parachains without changing defaults.
    let monitorChains =
        parseMonitorParachainsOverride() ?? monitorParams[env.name].TO_MONITOR_PARACHAINS
    if (monitorChains && monitorChains.length) {
        for (const paraid of monitorChains) {
            let chain = await context.parachain(paraid)
            let latestBlock = (await chain.query.system.number()).toPrimitive() as number
            let status = await subsquidV2.fetchSyncStatusOfParachain(
                context.graphqlApiUrl(),
                paraid,
            )
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
