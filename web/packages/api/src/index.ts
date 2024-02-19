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

export * as toPolkadot from './toPolkadot'
export * as toEthereum from './toEthereum'
export * as utils from './utils'
export * as status from './status'
