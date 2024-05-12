import { IERC20Metadata__factory, IERC20__factory } from '@snowbridge/contract-types'
import { Context } from './index'
import { fetchBeaconSlot } from './utils'

export type OperatingMode = 'Normal' | 'Halted'
export type BridgeStatusInfo = {
    toEthereum: {
        operatingMode: {
            outbound: OperatingMode,
        },
        latestPolkadotBlockOnEthereum: number,
        latestPolkaotBlock: number,
        blockLatency: number,
        latencySeconds: number,
    },
    toPolkadot: {
        operatingMode: {
            beacon: OperatingMode,
            inbound: OperatingMode,
            outbound: OperatingMode,
        },
        latestEthereumBlockOnPolkadot: number,
        latestEthereumBlock: number,
        blockLatency: number,
        latencySeconds: number,
    },
}
export type ChannelStatusInfo = {
    toEthereum: {
        outbound: number,
        inbound: number,
    },
    toPolkadot: {
        operatingMode: {
            outbound: OperatingMode
        },
        outbound: number,
        inbound: number,
    },
}

export const bridgeStatusInfo = async (context: Context, options = {
    polkadotBlockTimeInSeconds: 6,
    ethereumBlockTimeInSeconds: 12,
}): Promise<BridgeStatusInfo> => {
    const latestBeefyBlock = Number(await context.ethereum.contracts.beefyClient.latestBeefyBlock())
    const latestPolkadotBlock = (await context.polkadot.api.relaychain.query.system.number()).toPrimitive() as number

    const latestEthereumBlock = await context.ethereum.api.getBlockNumber()
    const latestBeaconBlockRoot = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.latestFinalizedBlockRoot()).toHex()
    let latestBeaconSlot = await fetchBeaconSlot(context.config.ethereum.beacon_url, latestBeaconBlockRoot)
    let latestBeaconExecutionBlock = latestBeaconSlot.data.message.body.execution_payload?.block_number
    while (latestBeaconExecutionBlock === undefined) {
        latestBeaconSlot = await fetchBeaconSlot(context.config.ethereum.beacon_url, latestBeaconSlot.data.message.slot - 1)
        latestBeaconExecutionBlock = latestBeaconSlot.data.message.body.execution_payload?.block_number
    }

    const beefyBlockLatency = latestPolkadotBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * options.polkadotBlockTimeInSeconds

    const beaconBlockLatency = latestEthereumBlock - Number(latestBeaconExecutionBlock)
    const beaconLatencySeconds = beaconBlockLatency * options.ethereumBlockTimeInSeconds

    const ethereumOperatingMode = await context.ethereum.contracts.gateway.operatingMode()
    const beaconOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.operatingMode()).toPrimitive()
    const inboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.operatingMode()).toPrimitive()
    const outboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.operatingMode()).toPrimitive()

    return {
        toEthereum: {
            operatingMode: {
                outbound: outboundOperatingMode as OperatingMode,
            },
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkaotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
        },
        toPolkadot: {
            operatingMode: {
                beacon: beaconOperatingMode as OperatingMode,
                inbound: inboundOperatingMode as OperatingMode,
                outbound: ethereumOperatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode,
            },
            latestEthereumBlockOnPolkadot: Number(latestBeaconExecutionBlock),
            latestEthereumBlock: latestEthereumBlock,
            blockLatency: beaconBlockLatency,
            latencySeconds: beaconLatencySeconds,
        },
    }
}

export const channelStatusInfo = async (context: Context, channelId: string): Promise<ChannelStatusInfo> => {
    const [inbound_nonce_eth, outbound_nonce_eth] = await context.ethereum.contracts.gateway.channelNoncesOf(channelId)
    const operatingMode = await context.ethereum.contracts.gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.nonce(channelId)).toPrimitive() as number
    const outbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.nonce(channelId)).toPrimitive() as number
    return {
        toEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
        },
        toPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
        },
    }
}
