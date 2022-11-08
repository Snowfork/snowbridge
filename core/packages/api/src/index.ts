import "@polkadot/api-augment/polkadot"

import { ethers } from "ethers"
import { ApiPromise, WsProvider } from "@polkadot/api"
import { H160 } from "@polkadot/types/interfaces"
import {
    DOTApp,
    DOTApp__factory,
    ETHApp,
    ETHApp__factory,
    ERC20App,
    ERC20App__factory
} from "@snowbridge/contracts"

interface Config {
    ethereum: {
        url: string
    }
    polkadot: {
        url: string
    }
}

interface AppContracts {
    dotApp: DOTApp
    ethApp: ETHApp
    erc20App: ERC20App
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

const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi = new ethers.providers.WebSocketProvider(config.ethereum.url)
    let polApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url)
    })

    let dotAppAddr = await polApi.query.dotApp.address<H160>()
    let ethAppAddr = await polApi.query.ethApp.address<H160>()
    let erc20AppAddr = await polApi.query.erc20App.address<H160>()

    let appContracts: AppContracts = {
        dotApp: DOTApp__factory.connect(dotAppAddr.toHex(), ethApi),
        ethApp: ETHApp__factory.connect(ethAppAddr.toHex(), ethApi),
        erc20App: ERC20App__factory.connect(erc20AppAddr.toHex(), ethApi)
    }

    let ethCtx = new EthereumContext(ethApi, appContracts)
    let polCtx = new PolkadotContext(polApi)

    return new Context(ethCtx, polCtx)
}
