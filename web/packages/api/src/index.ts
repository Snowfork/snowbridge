// import '@polkadot/api-augment/polkadot'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { ethers } from 'ethers'
import { BeefyClient, BeefyClient__factory, IGateway, IGateway__factory } from '@snowbridge/contract-types'
import { bnToU8a, stringToU8a, u8aToHex } from '@polkadot/util'

interface Config {
    ethereum: {
        url: string
    }
    polkadot: {
        url: {
            bridgeHub: string
            assetHub: string
            relaychain: string
        }
    }
    appContracts: {
        gateway: string
        beefy: string
    }
}

interface AppContracts {
    gateway: IGateway
    beefyClient: BeefyClient
}

export class Context {
    config: Config
    ethereum: EthereumContext
    polkadot: PolkadotContext

    constructor(config: Config, ethereum: EthereumContext, polkadot: PolkadotContext) {
        this.config = config
        this.ethereum = ethereum
        this.polkadot = polkadot
    }
}

class EthereumContext {
    api: ethers.WebSocketProvider
    contracts: AppContracts

    constructor(api: ethers.WebSocketProvider, contracts: AppContracts) {
        this.api = api
        this.contracts = contracts
    }
}

class PolkadotContext {
    api: {
        relaychain: ApiPromise
        assetHub: ApiPromise
        bridgeHub: ApiPromise
    }
    constructor(relaychain: ApiPromise, assetHub: ApiPromise, bridgeHub: ApiPromise) {
        this.api = {
            relaychain: relaychain,
            assetHub: assetHub,
            bridgeHub: bridgeHub,
        }
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi = new ethers.WebSocketProvider(config.ethereum.url)
    let relaychainApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.relaychain),
    })
    let assetHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.assetHub),
    })
    let bridgeHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.bridgeHub),
    })

    let gatewayAddr = config.appContracts.gateway
    let beefyAddr = config.appContracts.beefy

    let appContracts: AppContracts = {
        //TODO: Get gateway address from bridgehub
        gateway: IGateway__factory.connect(gatewayAddr, ethApi),
        //TODO: Get beefy client from gateway
        beefyClient: BeefyClient__factory.connect(beefyAddr, ethApi),
    }

    let ethCtx = new EthereumContext(ethApi, appContracts)
    let polCtx = new PolkadotContext(relaychainApi, assetHubApi, bridgeHubApi)

    return new Context(config, ethCtx, polCtx)
}

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

export const paraIdToSovereignAccount = (type: 'para' | 'sibl', paraId: number): string => {
    let typeEncoded = stringToU8a(type);
    let paraIdEncoded = bnToU8a(paraId);
    let zeroPadding = new Uint8Array(32 - typeEncoded.length - paraIdEncoded.length).fill(0);
    let address = new Uint8Array([...typeEncoded, ...paraIdEncoded, ...zeroPadding]);
    return u8aToHex(address)
}

export * as toPolkadot from './toPolkadot'
