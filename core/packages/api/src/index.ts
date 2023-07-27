import "@polkadot/api-augment/polkadot"
import { IGateway, IGateway__factory } from "@snowbridge/contract-types/types"

import { ethers } from "ethers"
import { ApiPromise, WsProvider } from "@polkadot/api"

interface Config {
    ethereum: {
        url: string
    }
    polkadot: {
        url: string
    }
}

interface AppContracts {
    gateway: IGateway
}

class Context {
    ethereum: EthereumContext
    polkadot: PolkadotContext

    constructor(ethereum: EthereumContext, polkadot: PolkadotContext) {
        this.ethereum = ethereum
        this.polkadot = polkadot
    }
}

class EthereumContext {
    api: ethers.providers.WebSocketProvider
    contracts: AppContracts

    constructor(api: ethers.providers.WebSocketProvider, contracts: AppContracts) {
        this.api = api
        this.contracts = contracts
    }
}

class PolkadotContext {
    api: ApiPromise
    constructor(api: ApiPromise) {
        this.api = api
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi = new ethers.providers.WebSocketProvider(config.ethereum.url)
    let polApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url),
    })

    let gatewayAddr = "" //address_for GatewayProxy

    let appContracts: AppContracts = {
        gateway: IGateway__factory.connect(gatewayAddr, ethApi),
    }

    let ethCtx = new EthereumContext(ethApi, appContracts)
    let polCtx = new PolkadotContext(polApi)

    return new Context(ethCtx, polCtx)
}
