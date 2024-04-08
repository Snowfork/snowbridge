import { IERC20__factory } from '@snowbridge/contract-types'
import { Context } from './index'

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

    const latestBeaconState = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.latestExecutionState()).toPrimitive() as { blockNumber: number }
    const latestEthereumBlock = await context.ethereum.api.getBlockNumber()

    const beefyBlockLatency = latestPolkadotBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * options.polkadotBlockTimeInSeconds

    const beaconBlockLatency = latestEthereumBlock - latestBeaconState.blockNumber
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
            latestEthereumBlockOnPolkadot: latestBeaconState.blockNumber,
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

export const assetStatusInfo = async (context: Context, tokenAddress: string, ownerAddress?: string) => {
    let [ethereumNetwork, gatewayAddress, isTokenRegistered] = await Promise.all([
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.getAddress(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])

    const ethereumChainId = ethereumNetwork.chainId
    const multiLocation = context.polkadot.api.assetHub.createType('StagingXcmV3MultiLocation', {
        parents: 2,
        interior: {
            X2: [
                { GlobalConsensus: { Ethereum: { chain_id: ethereumChainId } } },
                { AccountKey20: { key: tokenAddress } },
            ]
        }
    })
    const foreignAsset = (await context.polkadot.api.assetHub.query.foreignAssets.asset(multiLocation)).toPrimitive() as { status: 'Live' }

    const tokenContract = IERC20__factory.connect(tokenAddress, context.ethereum.api)
    let ownerBalance = BigInt(0)
    let tokenGatewayAllowance = BigInt(0)
    let tokenIsValidERC20 = true
    try {
        const owner = ownerAddress || "0x0000000000000000000000000000000000000000"
        const [tokenBalance_, tokenGatewayAllowance_] = await Promise.all([
            tokenContract.balanceOf(owner),
            tokenContract.allowance(owner, gatewayAddress),
        ])
        ownerBalance = tokenBalance_
        tokenGatewayAllowance = tokenGatewayAllowance_
    } catch {
        tokenIsValidERC20 = false
    }

    return {
        ethereumChainId,
        multiLocation,
        tokenIsValidERC20,
        tokenContract,
        isTokenRegistered,
        tokenGatewayAllowance,
        ownerBalance,
        foreignAssetExists: foreignAsset !== null,
        foreignAsset,
    }
}
