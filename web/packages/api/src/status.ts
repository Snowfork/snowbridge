import { Context, subsquidV2 } from "./index"
import { fetchBeaconSlot, fetchFinalityUpdate } from "./utils"
import { ApiPromise } from "@polkadot/api"
import { IGatewayV1, IGatewayV2 } from "@snowbridge/contract-types"

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
        v2Outbound?: number
        v2Inbound?: number
        // The estimated average delivery time for the most recent 10 messages.
        estimatedDeliveryTime?: number
        // The timeout duration of the oldest undelivered message.
        undeliveredTimeout?: number
    }
    toPolkadot: {
        operatingMode: {
            outbound: OperatingMode
        }
        outbound: number
        inbound: number
        v2Outbound?: number
        v2Inbound?: number
        estimatedDeliveryTime?: number
        undeliveredTimeout?: number
    }
}

type SourceType = "substrate" | "ethereum"

export type Sovereign = { name: string; account: string; balance: bigint; type: SourceType }

export type IndexerServiceStatusInfo = {
    chain: string
    latency: number
    id?: number
}

type Relayer = { name: string; account: string; type: SourceType; balance?: bigint }

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
    gateway: IGatewayV1 | IGatewayV2
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
    },
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
    const latestFinalizedBeefyBlock = (
        await relaychain.rpc.chain.getHeader(
            (await relaychain.rpc.beefy.getFinalizedHead()).toU8a(),
        )
    ).number.toNumber()
    const beefyBlockLatency = latestFinalizedBeefyBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * options.polkadotBlockTimeInSeconds

    // Beacon status
    const [latestFinalizedBeaconBlock, latestBeaconBlock] = await Promise.all([
        fetchFinalityUpdate(context.environment.beaconApiUrl),
        fetchBeaconSlot(context.environment.beaconApiUrl, "head"),
    ])
    const latestBeaconBlockRoot = (
        await bridgeHub.query.ethereumBeaconClient.latestFinalizedBlockRoot()
    ).toHex()
    const latestBeaconBlockOnPolkadot = Number(
        (await fetchBeaconSlot(context.environment.beaconApiUrl, latestBeaconBlockRoot)).data
            .message.slot,
    )
    const beaconBlockLatency =
        latestFinalizedBeaconBlock.finalized_slot - latestBeaconBlockOnPolkadot
    const beaconLatencySeconds = beaconBlockLatency * options.ethereumBlockTimeInSeconds

    // Operating mode
    const op = await getOperatingStatus({ gateway, bridgeHub })

    return {
        toEthereum: {
            operatingMode: op.toEthereum,
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkadotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
        },
        toPolkadot: {
            operatingMode: op.toPolkadot,
            latestBeaconSlotOnPolkadot: latestBeaconBlockOnPolkadot,
            latestBeaconSlotAttested: latestFinalizedBeaconBlock.attested_slot,
            latestBeaconSlotFinalized: latestFinalizedBeaconBlock.finalized_slot,
            latestBeaconSlotHead: latestBeaconBlock.data.message.slot,
            blockLatency: beaconBlockLatency,
            latencySeconds: beaconLatencySeconds,
        },
    }
}

export const ASSET_HUB_CHANNEL_ID =
    "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539"

export const channelStatusInfo = async (
    context: Context,
    channelId: string,
): Promise<ChannelStatusInfo> => {
    const [bridgeHub, gateway, gatewayV2] = await Promise.all([
        context.bridgeHub(),
        context.gateway(),
        context.gatewayV2(),
    ])

    // V1 nonces
    const [inbound_nonce_eth, outbound_nonce_eth] = await gateway.channelNoncesOf(channelId)
    const operatingMode = await gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (
        await bridgeHub.query.ethereumInboundQueue.nonce(channelId)
    ).toPrimitive() as number
    const outbound_nonce_sub = (
        await bridgeHub.query.ethereumOutboundQueue.nonce(channelId)
    ).toPrimitive() as number

    // V2 nonces
    const v2_outbound_nonce_eth = Number(await gatewayV2.v2_outboundNonce())
    const v2_outbound_nonce_sub = (
        await bridgeHub.query.ethereumOutboundQueueV2.nonce()
    ).toPrimitive() as number

    const v2_max_delivered_nonce_to_polkadot = await subsquidV2.fetchMaxDeliveredNonceToPolkadot(
        context.graphqlApiUrl(),
        v2_outbound_nonce_eth,
    )
    const v2_max_delivered_nonce_to_ethereum = await subsquidV2.fetchMaxDeliveredNonceToEthereum(
        context.graphqlApiUrl(),
        v2_outbound_nonce_sub,
    )

    let estimatedDeliveryTime: any,
        toEthereumUndeliveredTimeout: number | undefined,
        toPolkadotUndeliveredTimeout: number | undefined = undefined

    if (channelId.toLowerCase() == ASSET_HUB_CHANNEL_ID.toLowerCase()) {
        estimatedDeliveryTime = await subsquidV2.fetchEstimatedDeliveryTime(context.graphqlApiUrl())

        let latency = await subsquidV2.fetchToEthereumUndeliveredLatency(context.graphqlApiUrl())
        if (latency && latency.elapse) {
            toEthereumUndeliveredTimeout = latency.elapse
        }
        latency = await subsquidV2.fetchToPolkadotUndeliveredLatency(context.graphqlApiUrl())
        if (latency && latency.elapse) {
            toPolkadotUndeliveredTimeout = latency.elapse
        }
    }

    return {
        toEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
            v2Outbound: v2_outbound_nonce_sub,
            v2Inbound: v2_max_delivered_nonce_to_ethereum,
            estimatedDeliveryTime: estimatedDeliveryTime?.toEthereumV2Elapse?.elapse,
            undeliveredTimeout: toEthereumUndeliveredTimeout,
        },
        toPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? "Normal" : ("Halted" as OperatingMode),
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
            v2Outbound: v2_outbound_nonce_eth,
            v2Inbound: v2_max_delivered_nonce_to_polkadot,
            estimatedDeliveryTime: estimatedDeliveryTime?.toPolkadotV2Elapse?.elapse,
            undeliveredTimeout: toPolkadotUndeliveredTimeout,
        },
    }
}
