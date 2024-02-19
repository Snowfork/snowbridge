import { Context } from './index'

export type OperatingMode = 'Normal' | 'Halted'

export const bridgeStatusInfo = async (context: Context) => {
    const latestBeefyBlock = Number(await context.ethereum.contracts.beefyClient.latestBeefyBlock())
    const latestPolkadotBlock = (await context.polkadot.api.relaychain.query.system.number()).toPrimitive() as number

    const latestBeaconState = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.latestExecutionState()).toPrimitive() as { blockNumber: number }
    const latestEthereumBlock = await context.ethereum.api.getBlockNumber()

    const polkadotBlockTimeInSeconds = 6
    const beefyBlockLatency = latestPolkadotBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * polkadotBlockTimeInSeconds

    const ethereumBlockTimeInSeconds = 12
    const beaconBlockLatency = latestEthereumBlock - latestBeaconState.blockNumber
    const beaconLatencySeconds = beaconBlockLatency * ethereumBlockTimeInSeconds

    const ethereumOperatingMode = await context.ethereum.contracts.gateway.operatingMode()
    const beaconOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.operatingMode()).toPrimitive()
    const inboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.operatingMode()).toPrimitive()
    const outboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.operatingMode()).toPrimitive()

    return {
        polkadotToEthereum: {
            operatingMode: {
                outbound: outboundOperatingMode as OperatingMode,
            },
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkaotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
        },
        ethereumToPolkadot: {
            operatingMode: {
                beacon: beaconOperatingMode as OperatingMode,
                inbound: inboundOperatingMode as OperatingMode,
                outbound: ethereumOperatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode,
            },
            latestEthereumBlockOnPolkadot: latestBeaconState.blockNumber,
            latestEthereumBlock: latestEthereumBlock,
            blockLatency: beaconBlockLatency,
            latencySeconds: beaconLatencySeconds,
        },
    }
}

export const channelStatusInfo = async (context: Context, channelId: string) => {
    const [inbound_nonce_eth, outbound_nonce_eth] = await context.ethereum.contracts.gateway.channelNoncesOf(channelId)
    const operatingMode = await context.ethereum.contracts.gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.nonce(channelId)).toPrimitive() as number
    const outbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.nonce(channelId)).toPrimitive() as number
    return {
        ethereumToPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
        },
        polkadotToEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
        },
    }
}
