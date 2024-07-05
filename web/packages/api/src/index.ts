// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { AbstractProvider, JsonRpcProvider, WebSocketProvider } from "ethers"
import {
    BeefyClient,
    BeefyClient__factory,
    IGateway,
    IGateway__factory,
} from "@snowbridge/contract-types"

interface Config {
    ethereum: {
        execution_url: string | AbstractProvider
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
    indexApiUrl?: string
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
    api: AbstractProvider
    contracts: AppContracts

    constructor(api: AbstractProvider, contracts: AppContracts) {
        this.api = api
        this.contracts = contracts
    }
}

type Parachains = { [paraId: number]: ApiPromise }

class PolkadotContext {
    api: {
        relaychain: ApiPromise
        assetHub: ApiPromise
        bridgeHub: ApiPromise
        parachains: Parachains
    }
    constructor(
        relaychain: ApiPromise,
        assetHub: ApiPromise,
        bridgeHub: ApiPromise,
        parachains: Parachains
    ) {
        this.api = {
            relaychain: relaychain,
            assetHub: assetHub,
            bridgeHub: bridgeHub,
            parachains: parachains,
        }
    }
}

export const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi: AbstractProvider
    if (typeof config.ethereum.execution_url === "string") {
        if (config.ethereum.execution_url.startsWith("http")) {
            ethApi = new JsonRpcProvider(config.ethereum.execution_url)
        } else {
            ethApi = new WebSocketProvider(config.ethereum.execution_url)
        }
    } else {
        ethApi = config.ethereum.execution_url
    }

    const parasConnect: Promise<{ paraId: number; api: ApiPromise }>[] = []
    for (const parachain of config.polkadot.url.parachains ?? []) {
        parasConnect.push(addParachainConnection(parachain))
    }

    const [relaychainApi, assetHubApi, bridgeHubApi] = await Promise.all([
        ApiPromise.create({
            provider: config.polkadot.url.relaychain.startsWith("http")
                ? new HttpProvider(config.polkadot.url.relaychain)
                : new WsProvider(config.polkadot.url.relaychain),
        }),
        ApiPromise.create({
            provider: config.polkadot.url.assetHub.startsWith("http")
                ? new HttpProvider(config.polkadot.url.assetHub)
                : new WsProvider(config.polkadot.url.assetHub),
        }),
        ApiPromise.create({
            provider: config.polkadot.url.bridgeHub.startsWith("http")
                ? new HttpProvider(config.polkadot.url.bridgeHub)
                : new WsProvider(config.polkadot.url.bridgeHub),
        }),
    ])

    const paras = await Promise.all(parasConnect)
    const parachains: Parachains = {}
    for (const { paraId, api } of paras) {
        if (paraId in parachains) {
            throw new Error(`${paraId} already added.`)
        }
        parachains[paraId] = api
    }

    const gatewayAddr = config.appContracts.gateway
    const beefyAddr = config.appContracts.beefy

    const appContracts: AppContracts = {
        //TODO: Get gateway address from bridgehub
        gateway: IGateway__factory.connect(gatewayAddr, ethApi),
        //TODO: Get beefy client from gateway
        beefyClient: BeefyClient__factory.connect(beefyAddr, ethApi),
    }

    const ethCtx = new EthereumContext(ethApi, appContracts)
    const polCtx = new PolkadotContext(relaychainApi, assetHubApi, bridgeHubApi, parachains)

    const context = new Context(config, ethCtx, polCtx)
    await Promise.all(parasConnect)
    return context
}

export const addParachainConnection = async (url: string) => {
    const api = await ApiPromise.create({
        provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
    })
    const paraId = (await api.query.parachainInfo.parachainId()).toPrimitive() as number
    console.log(`${url} added with parachain id: ${paraId}`)
    return { paraId, api }
}

export const destroyContext = async (context: Context): Promise<void> => {
    // clean up etheruem
    await context.ethereum.contracts.beefyClient.removeAllListeners()
    await context.ethereum.contracts.gateway.removeAllListeners()
    if (typeof context.config.ethereum.execution_url === "string") {
        context.ethereum.api.destroy()
    }
    // clean up polkadot
    await context.polkadot.api.relaychain.disconnect()
    await context.polkadot.api.bridgeHub.disconnect()
    await context.polkadot.api.assetHub.disconnect()

    for (const paraId of Object.keys(context.polkadot.api.parachains)) {
        await context.polkadot.api.parachains[Number(paraId)].disconnect()
    }
}

export * as toPolkadot from "./toPolkadot"
export * as toEthereum from "./toEthereum"
export * as utils from "./utils"
export * as status from "./status"
export * as assets from "./assets"
export * as environment from "./environment"
export * as subscan from "./subscan"
export * as history from "./history"
