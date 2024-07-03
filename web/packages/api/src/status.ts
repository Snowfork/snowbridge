import { Context } from "./index"
import { fetchBeaconSlot, fetchFinalityUpdate } from "./utils"
import { Relayer, SourceType } from "./environment"

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
    }
    toPolkadot: {
        operatingMode: {
            outbound: OperatingMode
        }
        outbound: number
        inbound: number
        previousOutbound: number
        previousInbound: number
    }
}

export enum AlarmReason {
    BeefyStale = "BeefyStale",
    BeaconStale = "BeaconStale",
    ToEthereumChannelStale = "ToEthereumChannelStale",
    ToPolkadotChannelStale = "ToPolkadotChannelStale",
    AccountBalanceInsufficient = "AccountBalanceInsufficient",
}

export type Sovereign = { name: string; account: string; balance: bigint; type: SourceType }

export const BlockLatencyThreshold = {
    // Syncing beefy finality update every 4 hours(2400 blocks) so we set 3000 blocks at most.
    ToEthereum: 3000,
    // Syncing beacon finality update every 6.4 minutes(32 blocks) so we set 128 blocks (4 epochs) at most.
    ToPolkadot: 128,
}

export const InsufficientBalanceThreshold = {
    // Minimum as 300 DOT
    Substrate: 3_000_000_000_000,
    // Minimum as 0.3 Ether
    Ethereum: 300_000_000_000_000_000,
}

export type AllMetrics = {
    name: string
    bridgeStatus: BridgeStatusInfo
    channels: ChannelStatusInfo[]
    sovereigns: Sovereign[]
    relayers: Relayer[]
}

export const bridgeStatusInfo = async (
    context: Context,
    options = {
        polkadotBlockTimeInSeconds: 6,
        ethereumBlockTimeInSeconds: 12,
        toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
        toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
    }
): Promise<BridgeStatusInfo> => {
    // Beefy status
    const latestBeefyBlock = Number(await context.ethereum.contracts.beefyClient.latestBeefyBlock())
    const latestPolkadotBlock = (
        await context.polkadot.api.relaychain.query.system.number()
    ).toPrimitive() as number
    const latestBeaconSlot = await context.ethereum.api.getBlockNumber()
    const latestFinalizedBeefyBlock = (
        await context.polkadot.api.relaychain.rpc.chain.getHeader(
            (await context.polkadot.api.relaychain.rpc.beefy.getFinalizedHead()).toU8a()
        )
    ).number.toNumber()
    const beefyBlockLatency = latestFinalizedBeefyBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * options.polkadotBlockTimeInSeconds
    const previousBeefyBlock = Number(
        await context.ethereum.contracts.beefyClient.latestBeefyBlock({
            blockTag:
                latestBeaconSlot > options.toEthereumCheckIntervalInBlock
                    ? latestBeaconSlot - options.toEthereumCheckIntervalInBlock
                    : 100,
        })
    )

    // Beacon status
    const latestFinalizedBeaconBlock = await fetchFinalityUpdate(context.config.ethereum.beacon_url)
    const latestBeaconBlockRoot = (
        await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.latestFinalizedBlockRoot()
    ).toHex()
    const latestBeaconBlockOnPolkadot = Number(
        (await fetchBeaconSlot(context.config.ethereum.beacon_url, latestBeaconBlockRoot)).data
            .message.slot
    )
    const beaconBlockLatency =
        latestFinalizedBeaconBlock.attested_slot - latestBeaconBlockOnPolkadot
    const beaconLatencySeconds = beaconBlockLatency * options.ethereumBlockTimeInSeconds
    const latestBridgeHubBlock = (
        await context.polkadot.api.bridgeHub.query.system.number()
    ).toPrimitive() as number
    const previousBridgeHubBlock = await context.polkadot.api.bridgeHub.query.system.blockHash(
        latestBridgeHubBlock > options.toPolkadotCheckIntervalInBlock
            ? latestBridgeHubBlock - options.toPolkadotCheckIntervalInBlock
            : 10
    )
    const bridgeHubApiAt = await context.polkadot.api.bridgeHub.at(previousBridgeHubBlock.toU8a())
    const previousBeaconBlockRoot =
        await bridgeHubApiAt.query.ethereumBeaconClient.latestFinalizedBlockRoot()
    const previousBeaconBlock = Number(
        (await fetchBeaconSlot(context.config.ethereum.beacon_url, previousBeaconBlockRoot.toHex()))
            .data.message.slot
    )

    // Operating mode
    const ethereumOperatingMode = await context.ethereum.contracts.gateway.operatingMode()
    const beaconOperatingMode = (
        await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.operatingMode()
    ).toPrimitive()
    const inboundOperatingMode = (
        await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.operatingMode()
    ).toPrimitive()
    const outboundOperatingMode = (
        await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.operatingMode()
    ).toPrimitive()

    return {
        toEthereum: {
            operatingMode: {
                outbound: outboundOperatingMode as OperatingMode,
            },
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkadotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
            previousPolkadotBlockOnEthereum: previousBeefyBlock,
        },
        toPolkadot: {
            operatingMode: {
                beacon: beaconOperatingMode as OperatingMode,
                inbound: inboundOperatingMode as OperatingMode,
                outbound: ethereumOperatingMode === 0n ? "Normal" : ("Halted" as OperatingMode),
            },
            latestBeaconSlotOnPolkadot: latestBeaconBlockOnPolkadot,
            latestBeaconSlotAttested: latestFinalizedBeaconBlock.attested_slot,
            latestBeaconSlotFinalized: latestFinalizedBeaconBlock.finalized_slot,
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
        toPolkadotCheckIntervalInBlock: BlockLatencyThreshold.ToEthereum,
        toEthereumCheckIntervalInBlock: BlockLatencyThreshold.ToPolkadot,
    }
): Promise<ChannelStatusInfo> => {
    const [inbound_nonce_eth, outbound_nonce_eth] =
        await context.ethereum.contracts.gateway.channelNoncesOf(channelId)
    const operatingMode = await context.ethereum.contracts.gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (
        await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.nonce(channelId)
    ).toPrimitive() as number
    const outbound_nonce_sub = (
        await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.nonce(channelId)
    ).toPrimitive() as number

    const latestEthereumBlock = await context.ethereum.api.getBlockNumber()
    const [previous_inbound_nonce_eth, previous_outbound_nonce_eth] =
        await context.ethereum.contracts.gateway.channelNoncesOf(channelId, {
            blockTag:
                latestEthereumBlock > options.toEthereumCheckIntervalInBlock
                    ? latestEthereumBlock - options.toEthereumCheckIntervalInBlock
                    : 100,
        })
    const latestBridgeHubBlock = (
        await context.polkadot.api.bridgeHub.query.system.number()
    ).toPrimitive() as number
    const previousBridgeHubBlock = await context.polkadot.api.bridgeHub.query.system.blockHash(
        latestBridgeHubBlock > options.toPolkadotCheckIntervalInBlock
            ? latestBridgeHubBlock - options.toPolkadotCheckIntervalInBlock
            : 10
    )
    const bridgeHubApiAt = await context.polkadot.api.bridgeHub.at(previousBridgeHubBlock.toU8a())
    const previous_inbound_nonce_sub = (
        await bridgeHubApiAt.query.ethereumInboundQueue.nonce(channelId)
    ).toPrimitive() as number
    const previous_outbound_nonce_sub = (
        await bridgeHubApiAt.query.ethereumOutboundQueue.nonce(channelId)
    ).toPrimitive() as number

    return {
        toEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
            previousOutbound: previous_outbound_nonce_sub,
            previousInbound: Number(previous_inbound_nonce_eth),
        },
        toPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? "Normal" : ("Halted" as OperatingMode),
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
            previousOutbound: Number(previous_outbound_nonce_eth),
            previousInbound: previous_inbound_nonce_sub,
        },
    }
}
