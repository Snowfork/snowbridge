"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@polkadot/api-augment/polkadot");
const ethers_1 = require("ethers");
const api_1 = require("@polkadot/api");
const contracts_1 = require("@snowbridge/contracts");
class Context {
    constructor(ethereum, polkadot) {
        this.ethereum = ethereum;
        this.polkadot = polkadot;
    }
}
class EthereumContext {
    constructor(api, contracts) {
        this.api = api;
        this.contracts = contracts;
    }
}
class PolkadotContext {
    constructor(api) {
        this.api = api;
    }
}
const contextFactory = async (config) => {
    let ethApi = new ethers_1.ethers.providers.WebSocketProvider(config.ethereum.url);
    let polApi = await api_1.ApiPromise.create({
        provider: new api_1.WsProvider(config.polkadot.url)
    });
    let dotAppAddr = await polApi.query.dotApp.address();
    let ethAppAddr = await polApi.query.ethApp.address();
    let erc20AppAddr = await polApi.query.erc20App.address();
    let appContracts = {
        dotApp: contracts_1.DOTApp__factory.connect(dotAppAddr.toHex(), ethApi),
        ethApp: contracts_1.ETHApp__factory.connect(ethAppAddr.toHex(), ethApi),
        erc20App: contracts_1.ERC20App__factory.connect(erc20AppAddr.toHex(), ethApi)
    };
    let ethCtx = new EthereumContext(ethApi, appContracts);
    let polCtx = new PolkadotContext(polApi);
    return new Context(ethCtx, polCtx);
};
