import { Context } from "./index"
import { fetchBeaconSlot, fetchFinalityUpdate } from "./utils"
import { fetchEstimatedDeliveryTime } from "./subsquid"
import { Relayer, SourceType } from "./environment"
import { ApiPromise } from "@polkadot/api"
import { IGateway } from "@snowbridge/contract-types"

export type OperatingMode = "Normal" | "Halted"
export type BridgeStatusInfo = {
    toEthereum: {
        operatingMode: {
            outbound: OperatingMode
        }
        latestPolkadotBlockOnEthereum: number
        latestPolkadotBlock: number
        blockLatency: number
        latencySeconds: number
        previousPolkadotBlockOnEthereum: number
    }
    toPolkadot: {
        operatingMode: {
            beacon: OperatingMode
            inbound: OperatingMode
            outbound: OperatingMode
        }
        latestBeaconSlotOnPolkadot: number
        latestBeaconSlotAttested: number
        latestBeaconSlotFinalized: number
        latestBeaconSlotHead: number
        blockLatency: number
        latencySeconds: number
        previousEthereumBlockOnPolkadot: number
    }
}

export enum ChannelKind {
    Primary = "Primary",
    Secondary = "Secondary",
    AssetHub = "AssetHub",
}

export type ChannelStatusInfo = {
    name?: ChannelKind
    toEthereum: {
        outbound: number
        inbound: number
        previousOutbound: number
        previousInbound: number
        estimatedDeliveryTime?: number
    }
    toPolkadot: {
        operatingMode: {
            outbound: OperatingMode
        }
        outbound: number
        inbound: number
        previousOutbound: number
        previousInbound: number
        estimatedDeliveryTime?: number
    }
}

export type Sovereign = { name: string; account: string; balance: bigint; type: SourceType }

export type IndexerServiceStatusInfo = {
    chain: string
    latency: number
}

export type AllMetrics = {
    name: string
    bridgeStatus: BridgeStatusInfo
    channels: ChannelStatusInfo[]
    sovereigns: Sovereign[]
    relayers: Relayer[]
    indexerStatus: IndexerServiceStatusInfo[]
}

export type OperationStatus = {
    toEthereum: {
        outbound: OperatingMode
    }
    toPolkadot: {
        beacon: OperatingMode
        inbound: OperatingMode
        outbound: OperatingMode
    }
}
export async function getOperatingStatus({
    gateway,
    bridgeHub,
}: {
    gateway: IGateway
    bridgeHub: ApiPromise
}): Promise<OperationStatus> {
    const ethereumOperatingMode = await gateway.operatingMode()
    const beaconOperatingMode = (
        await bridgeHub.query.ethereumBeaconClient.operatingMode()
    ).toPrimitive()
    const inboundOperatingMode = (
        await bridgeHub.query.ethereumInboundQueue.operatingMode()
    ).toPrimitive()
    const outboundOperatingMode = (
        await bridgeHub.query.ethereumOutboundQueue.operatingMode()
    ).toPrimitive()

    return {
        toEthereum: {
            outbound: outboundOperatingMode as OperatingMode,
        },
        toPolkadot: {
            beacon: beaconOperatingMode as OperatingMode,
            inbound: inboundOperatingMode as OperatingMode,
            outbound: ethereumOperatingMode === 0n ? "Normal" : ("Halted" as OperatingMode),
        },
    }
}

export const bridgeStatusInfo = async (
    context: Context,
    options = {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
        toPolkadotCheckIntervalInBlock: 120,
        toEthereumCheckIntervalInBlock: 2400,
    }
): Promise<BridgeStatusInfo> => {
    const [bridgeHub, ethereum, gateway, beefyClient, relaychain] = await Promise.all([
        context.bridgeHub(),
        context.ethereum(),
        context.gateway(),
        context.beefyClient(),
        context.relaychain(),
    ])

    // Beefy status
    const latestBeefyBlock = Number(await beefyClient.latestBeefyBlock())
    const latestPolkadotBlock = (await relaychain.query.system.number()).toPrimitive() as number
    const latestBeaconSlot = await ethereum.getBlockNumber()
    const latestFinalizedBeefyBlock = (
        await relaychain.rpc.chain.getHeader(
            (await relaychain.rpc.beefy.getFinalizedHead()).toU8a()
        )
    ).number.toNumber()
    const beefyBlockLatency = latestFinalizedBeefyBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * options.polkadotBlockTimeInSeconds
    const previousBeefyBlock = Number(
        await beefyClient.latestBeefyBlock({
            blockTag:
                latestBeaconSlot > options.toEthereumCheckIntervalInBlock
                    ? latestBeaconSlot - options.toEthereumCheckIntervalInBlock
                    : 100,
        })
    )

    // Beacon status
    const [latestFinalizedBeaconBlock, latestBeaconBlock] = await Promise.all([
        fetchFinalityUpdate(context.config.ethereum.beacon_url),
        fetchBeaconSlot(context.config.ethereum.beacon_url, "head"),
    ])
    const latestBeaconBlockRoot = (
        await bridgeHub.query.ethereumBeaconClient.latestFinalizedBlockRoot()
    ).toHex()
    const latestBeaconBlockOnPolkadot = Number(
        (await fetchBeaconSlot(context.config.ethereum.beacon_url, latestBeaconBlockRoot)).data
            .message.slot
    )
    const beaconBlockLatency = latestBeaconBlock.data.message.slot - latestBeaconBlockOnPolkadot
    const beaconLatencySeconds = beaconBlockLatency * options.ethereumBlockTimeInSeconds
    const latestBridgeHubBlock = (await bridgeHub.query.system.number()).toPrimitive() as number
    const previousBridgeHubBlock = await bridgeHub.query.system.blockHash(
        latestBridgeHubBlock > options.toPolkadotCheckIntervalInBlock
            ? latestBridgeHubBlock - options.toPolkadotCheckIntervalInBlock
            : 10
    )
    const bridgeHubApiAt = await bridgeHub.at(previousBridgeHubBlock.toU8a())
    const previousBeaconBlockRoot =
        await bridgeHubApiAt.query.ethereumBeaconClient.latestFinalizedBlockRoot()
    const previousBeaconBlock = Number(
        (await fetchBeaconSlot(context.config.ethereum.beacon_url, previousBeaconBlockRoot.toHex()))
            .data.message.slot
    )

    // Operating mode
    const op = await getOperatingStatus({ gateway, bridgeHub })

    return {
        toEthereum: {
            operatingMode: op.toEthereum,
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkadotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
            previousPolkadotBlockOnEthereum: previousBeefyBlock,
        },
        toPolkadot: {
            operatingMode: op.toPolkadot,
            latestBeaconSlotOnPolkadot: latestBeaconBlockOnPolkadot,
            latestBeaconSlotAttested: latestFinalizedBeaconBlock.attested_slot,
            latestBeaconSlotFinalized: latestFinalizedBeaconBlock.finalized_slot,
            latestBeaconSlotHead: latestBeaconBlock.data.message.slot,
            blockLatency: beaconBlockLatency,
            latencySeconds: beaconLatencySeconds,
            previousEthereumBlockOnPolkadot: previousBeaconBlock,
        },
    }
}

export const channelStatusInfo = async (
    context: Context,
    channelId: string,
    options = {
        toPolkadotCheckIntervalInBlock: 120,
        toEthereumCheckIntervalInBlock: 2400,
    }
): Promise<ChannelStatusInfo> => {
    const [bridgeHub, ethereum, gateway] = await Promise.all([
        context.bridgeHub(),
        context.ethereum(),
        context.gateway(),
    ])

    const [inbound_nonce_eth, outbound_nonce_eth] = await gateway.channelNoncesOf(channelId)
    const operatingMode = await gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (
        await bridgeHub.query.ethereumInboundQueue.nonce(channelId)
    ).toPrimitive() as number
    const outbound_nonce_sub = (
        await bridgeHub.query.ethereumOutboundQueue.nonce(channelId)
    ).toPrimitive() as number

    const latestEthereumBlock = await ethereum.getBlockNumber()
    const [previous_inbound_nonce_eth, previous_outbound_nonce_eth] = await gateway.channelNoncesOf(
        channelId,
        {
            blockTag:
                latestEthereumBlock > options.toEthereumCheckIntervalInBlock
                    ? latestEthereumBlock - options.toEthereumCheckIntervalInBlock
                    : 100,
        }
    )
    const latestBridgeHubBlock = (await bridgeHub.query.system.number()).toPrimitive() as number
    const previousBridgeHubBlock = await bridgeHub.query.system.blockHash(
        latestBridgeHubBlock > options.toPolkadotCheckIntervalInBlock
            ? latestBridgeHubBlock - options.toPolkadotCheckIntervalInBlock
            : 10
    )
    const bridgeHubApiAt = await bridgeHub.at(previousBridgeHubBlock.toU8a())
    const previous_inbound_nonce_sub = (
        await bridgeHubApiAt.query.ethereumInboundQueue.nonce(channelId)
    ).toPrimitive() as number
    const previous_outbound_nonce_sub = (
        await bridgeHubApiAt.query.ethereumOutboundQueue.nonce(channelId)
    ).toPrimitive() as number

    let estimatedDeliveryTime: any
    if (
        context.config.graphqlApiUrl &&
        channelId.toLowerCase() ==
            "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539"
    ) {
        try {
            estimatedDeliveryTime = await fetchEstimatedDeliveryTime(channelId)
        } catch (e: any) {
            console.error("estimate api error:" + e.message)
        }
    }

    return {
        toEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
            previousOutbound: previous_outbound_nonce_sub,
            previousInbound: Number(previous_inbound_nonce_eth),
            estimatedDeliveryTime: Math.ceil(
                Number(estimatedDeliveryTime?.toEthereumElapse?.elapse)
            ),
        },
        toPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? "Normal" : ("Halted" as OperatingMode),
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
            previousOutbound: Number(previous_outbound_nonce_eth),
            previousInbound: previous_inbound_nonce_sub,
            estimatedDeliveryTime: Math.ceil(
                Number(estimatedDeliveryTime?.toPolkadotElapse?.elapse)
            ),
        },
    }
}
