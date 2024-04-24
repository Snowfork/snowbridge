// import '@polkadot/api-augment/polkadot'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { AbstractProvider, ethers } from 'ethers'
import { BeefyClient, BeefyClient__factory, IGateway, IGateway__factory } from '@snowbridge/contract-types'

interface Config {
    ethereum: {
        execution_url: string
        beacon_url: string
    }
    polkadot: {
        url: {
            bridgeHub: string
            assetHub: string
            relaychain: string
            parachains?: string[]
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
    api: ethers.AbstractProvider
    contracts: AppContracts

    constructor(api: ethers.AbstractProvider, contracts: AppContracts) {
        this.api = api
        this.contracts = contracts
    }
}

class PolkadotContext {
    api: {
        relaychain: ApiPromise
        assetHub: ApiPromise
        bridgeHub: ApiPromise
        parachains: { [paraId: number]: ApiPromise }
    }
    constructor(relaychain: ApiPromise, assetHub: ApiPromise, bridgeHub: ApiPromise) {
        this.api = {
            relaychain: relaychain,
            assetHub: assetHub,
            bridgeHub: bridgeHub,
            parachains: {}
        }
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi: AbstractProvider;
    if(config.ethereum.execution_url.startsWith("http")) {
        ethApi = new ethers.JsonRpcProvider(config.ethereum.execution_url)
    } else {
        ethApi = new ethers.WebSocketProvider(config.ethereum.execution_url)
    }
    const relaychainApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.relaychain),
    })
    const assetHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.assetHub),
    })
    const bridgeHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.bridgeHub),
    })

    const gatewayAddr = config.appContracts.gateway
    const beefyAddr = config.appContracts.beefy

    const appContracts: AppContracts = {
        //TODO: Get gateway address from bridgehub
        gateway: IGateway__factory.connect(gatewayAddr, ethApi),
        //TODO: Get beefy client from gateway
        beefyClient: BeefyClient__factory.connect(beefyAddr, ethApi),
    }

    const ethCtx = new EthereumContext(ethApi, appContracts)
    const polCtx = new PolkadotContext(relaychainApi, assetHubApi, bridgeHubApi)

    const context = new Context(config, ethCtx, polCtx)
    for (const parachain of config.polkadot.url.parachains ?? []) {
        await addParachainConnection(context, parachain)
    }
    return context
}

export const addParachainConnection = async (context: Context, url: string): Promise<void> => {
    const api = await ApiPromise.create({
        provider: new WsProvider(url)
    })
    const paraId = (await api.query.parachainInfo.parachainId()).toPrimitive() as number
    if (paraId in context.polkadot.api.parachains) {
        throw new Error(`${paraId} already added.`)
    }
    context.polkadot.api.parachains[paraId] = api
    console.log(`${url} added with parachain id: ${paraId}`)
}

export const destroyContext = async (context: Context): Promise<void> => {
    // clean up etheruem
    await context.ethereum.contracts.beefyClient.removeAllListeners()
    await context.ethereum.contracts.gateway.removeAllListeners()
    await context.ethereum.api.destroy()
    // clean up polkadot
    await context.polkadot.api.relaychain.disconnect()
    await context.polkadot.api.bridgeHub.disconnect()
    await context.polkadot.api.assetHub.disconnect()
    
    for (const paraId of Object.keys(context.polkadot.api.parachains)) {
        await context.polkadot.api.parachains[Number(paraId)].disconnect()
    }
}

export * as toPolkadot from './toPolkadot'
export * as toEthereum from './toEthereum'
export * as utils from './utils'
export * as status from './status'
