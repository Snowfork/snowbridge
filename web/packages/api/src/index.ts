// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { AbstractProvider, Contract, Interface, InterfaceAbi } from "ethers"
import {
    BeefyClient,
    IGatewayV1,
    IGatewayV2,
    ISwapLegacyRouter,
    ISwapQuoter,
    ISwapRouter,
    SnowbridgeL1Adaptor,
    SnowbridgeL2Adaptor,
} from "./contracts"
import { EthersEthereumProvider, type EthereumProvider } from "./EthereumProvider"
import { BridgeInfo, Environment } from "@snowbridge/base-types"

export * as toPolkadotV2 from "./toPolkadot_v2"
export * as toEthereumV2 from "./toEthereum_v2"
export * as utils from "./utils"
export * as status from "./status"
export * as assetsV2 from "./assets_v2"
export * as history from "./history"
export * as historyV2 from "./history_v2"
export * as subsquidV2 from "./subsquid_v2"
export * as forKusama from "./forKusama"
export * as forInterParachain from "./forInterParachain"
export * as toEthereumFromEVMV2 from "./toEthereumFromEVM_v2"
export * as parachains from "./parachains"
export * as xcmBuilder from "./xcmBuilder"
export * as toEthereumSnowbridgeV2 from "./toEthereumSnowbridgeV2"
export * as neuroWeb from "./parachains/neuroweb"
export * as toPolkadotSnowbridgeV2 from "./toPolkadotSnowbridgeV2"
export * as addTip from "./addTip"
export { EthersEthereumProvider } from "./EthereumProvider"
export type { EthereumProvider } from "./EthereumProvider"

export class Context {
    readonly environment: Environment
    readonly ethereumProvider: EthereumProvider<AbstractProvider, Contract, InterfaceAbi, Interface>

    // Ethereum
    #ethChains: Record<string, AbstractProvider>
    #gateway?: Contract & IGatewayV1
    #gatewayV2?: Contract & IGatewayV2
    #beefyClient?: Contract & BeefyClient
    #l1Adapter?: Contract & SnowbridgeL1Adaptor
    #l1SwapQuoter?: Contract & ISwapQuoter
    #l1SwapRouter?: Contract & ISwapRouter
    #l1LegacySwapRouter?: Contract & ISwapLegacyRouter
    #l2Adapters: { [l2ChainId: number]: Contract & SnowbridgeL2Adaptor } = {}

    // Substrate
    #polkadotParachains: Record<string, Promise<ApiPromise>>
    #kusamaParachains: Record<string, Promise<ApiPromise>>
    #relaychain?: ApiPromise

    static #rpcInitTimeoutMs = 40_000
    static #wsRequestTimeoutMs = 30_000

    constructor(environment: Environment) {
        this.environment = environment
        this.ethereumProvider = new EthersEthereumProvider()
        this.#polkadotParachains = {}
        this.#kusamaParachains = {}
        this.#ethChains = {}
    }

    async #createApi(options: {
        provider: HttpProvider | WsProvider
        noInitWarn?: boolean
        types?: any
    }): Promise<ApiPromise> {
        let timer: NodeJS.Timeout | undefined
        try {
            const timeoutPromise = new Promise<never>((_, reject) => {
                timer = setTimeout(() => {
                    reject(new Error(`Api init timed out after ${Context.#rpcInitTimeoutMs}ms`))
                }, Context.#rpcInitTimeoutMs)
            })
            return await Promise.race([ApiPromise.create(options), timeoutPromise])
        } finally {
            if (timer) clearTimeout(timer)
        }
    }

    #buildProvider(url: string) {
        if (url.startsWith("http")) {
            return new HttpProvider(url)
        }
        return new WsProvider(url, undefined, {}, Context.#wsRequestTimeoutMs)
    }

    async relaychain(): Promise<ApiPromise> {
        if (this.#relaychain) {
            return this.#relaychain
        }
        const url = this.environment.relaychainUrl
        console.log("Connecting to the relaychain.")
        this.#relaychain = await this.#createApi({
            noInitWarn: true,
            provider: this.#buildProvider(url),
        })
        console.log("Connected to the relaychain.")
        return this.#relaychain
    }

    assetHub(): Promise<ApiPromise> {
        return this.parachain(this.environment.assetHubParaId)
    }

    kusamaAssetHub(): Promise<ApiPromise> {
        if (!this.environment.kusama) {
            throw Error(`Kusama config is not set.`)
        }
        const assetHubParaId = this.environment.kusama.assetHubParaId
        return this.kusamaParachain(assetHubParaId)
    }

    bridgeHub(): Promise<ApiPromise> {
        return this.parachain(this.environment.bridgeHubParaId)
    }

    hasParachain(paraId: number): boolean {
        return paraId.toString() in this.environment.parachains
    }

    hasEthChain(ethChainId: number): boolean {
        return ethChainId.toString() in this.environment.ethereumChains
    }

    parachains(): number[] {
        return Object.keys(this.environment.parachains).map((key) => Number(key))
    }

    ethChains(): number[] {
        return Object.keys(this.environment.ethereumChains).map((key) => Number(key))
    }

    async parachain(paraId: number): Promise<ApiPromise> {
        const paraIdKey = paraId.toString()
        if (paraIdKey in this.#polkadotParachains) {
            return await this.#polkadotParachains[paraIdKey]
        }
        this.#polkadotParachains[paraIdKey] = new Promise((resolve, reject) => {
            const { parachains } = this.environment
            if (paraIdKey in parachains) {
                const url = parachains[paraIdKey]
                let options: any = {
                    noInitWarn: true,
                    provider: this.#buildProvider(url),
                }
                if (paraId === this.environment.bridgeHubParaId) {
                    options.types = {
                        ContractCall: {
                            target: "[u8; 20]",
                            calldata: "Vec<u8>",
                            value: "u128",
                            gas: "u64",
                        },
                    }
                }
                console.log("Connecting to parachain", paraIdKey, url)
                this.#createApi(options)
                    .then((a) => {
                        console.log("Connected to parachain", paraIdKey)
                        resolve(a)
                    })
                    .catch((error) => {
                        delete this.#polkadotParachains[paraIdKey]
                        reject(error)
                    })
            } else {
                reject(Error(`Parachain id ${paraId} not in the list of parachain urls.`))
            }
        })
        return await this.#polkadotParachains[paraIdKey]
    }

    async kusamaParachain(paraId: number): Promise<ApiPromise> {
        const paraIdKey = paraId.toString()
        if (paraIdKey in this.#kusamaParachains) {
            return await this.#kusamaParachains[paraIdKey]
        }
        this.#kusamaParachains[paraIdKey] = new Promise((resolve, reject) => {
            if (!this.environment.kusama) {
                reject(Error(`Kusama config is not set.`))
                return
            }
            const { parachains } = this.environment.kusama
            if (paraIdKey in parachains) {
                const url = parachains[paraIdKey]
                const options = {
                    noInitWarn: true,
                    provider: this.#buildProvider(url),
                }
                console.log("Connecting to Kusama parachain", paraIdKey, url)
                this.#createApi(options)
                    .then((a) => {
                        console.log("Connected to Kusama parachain", paraIdKey)
                        resolve(a)
                    })
                    .catch((error) => {
                        delete this.#kusamaParachains[paraIdKey]
                        reject(error)
                    })
            } else {
                reject(Error(`Parachain id ${paraId} not in the list of parachain urls.`))
            }
        })
        return await this.#kusamaParachains[paraIdKey]
    }

    setEthProvider(ethChainId: number, provider: AbstractProvider) {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            this.ethereumProvider.destroyProvider(this.#ethChains[ethChainKey])
        }
        this.#ethChains[ethChainKey] = provider
    }

    ethChain(ethChainId: number): AbstractProvider {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            return this.#ethChains[ethChainKey]
        }

        const { ethereumChains } = this.environment
        if (ethChainKey in ethereumChains) {
            const url = ethereumChains[ethChainKey]
            let provider: AbstractProvider
            if (typeof url === "string") {
                provider = this.ethereumProvider.createProvider(url)
            } else {
                provider = url as AbstractProvider
            }
            this.#ethChains[ethChainKey] = provider
            return provider
        } else {
            throw Error(`Ethereum chain id ${ethChainKey} not in the list of ethereum urls.`)
        }
    }

    ethereum(): AbstractProvider {
        return this.ethChain(this.environment.ethChainId)
    }

    gateway(): Contract & IGatewayV1 {
        if (this.#gateway) {
            return this.#gateway
        }
        this.#gateway = this.ethereumProvider.connectGatewayV1(
            this.environment.gatewayContract,
            this.ethereum(),
        )
        return this.#gateway
    }

    gatewayV2(): Contract & IGatewayV2 {
        if (this.#gatewayV2) {
            return this.#gatewayV2
        }
        this.#gatewayV2 = this.ethereumProvider.connectGatewayV2(
            this.environment.gatewayContract,
            this.ethereum(),
        )
        return this.#gatewayV2
    }

    beefyClient(): Contract & BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        this.#beefyClient = this.ethereumProvider.connectBeefyClient(
            this.environment.beefyContract,
            this.ethereum(),
        )
        return this.#beefyClient
    }

    graphqlApiUrl(): string {
        return this.environment.indexerGraphQlUrl
    }

    async destroyContext(): Promise<void> {
        // clean up contract listeners
        if (this.#beefyClient) {
            await this.ethereumProvider.destroyContract(this.beefyClient())
        }
        if (this.#gateway) {
            await this.ethereumProvider.destroyContract(this.gateway())
        }
        if (this.#gatewayV2) {
            await this.ethereumProvider.destroyContract(this.gatewayV2())
        }

        // clean up ethereum
        for (const ethChainKey of Object.keys(this.environment.ethereumChains)) {
            if (
                typeof this.environment.ethereumChains[ethChainKey] === "string" &&
                this.#ethChains[ethChainKey]
            ) {
                this.ethereumProvider.destroyProvider(this.#ethChains[ethChainKey])
            }
        }
        // clean up polkadot
        if (this.#relaychain) {
            await this.#relaychain.disconnect()
        }

        for (const paraId of Object.keys(this.#polkadotParachains)) {
            await (await this.#polkadotParachains[Number(paraId)]).disconnect()
        }
        for (const paraId of Object.keys(this.#kusamaParachains)) {
            await (await this.#kusamaParachains[Number(paraId)]).disconnect()
        }
    }

    l1Adapter(): Contract & SnowbridgeL1Adaptor {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1Adapter) {
            return this.#l1Adapter
        }
        this.#l1Adapter = this.ethereumProvider.connectL1Adapter(
            this.environment.l2Bridge.l1AdapterAddress as string,
            this.ethereum(),
        )
        return this.#l1Adapter
    }

    l1SwapQuoter(): Contract & ISwapQuoter {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1SwapQuoter) {
            return this.#l1SwapQuoter
        }
        this.#l1SwapQuoter = this.ethereumProvider.connectL1SwapQuoter(
            this.environment.l2Bridge.l1SwapQuoterAddress as string,
            this.ethereum(),
        )
        return this.#l1SwapQuoter
    }

    l1SwapRouterAddress(): string {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        return this.environment.l2Bridge.l1SwapRouterAddress as string
    }

    l1SwapRouter(): Contract & ISwapRouter {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1SwapRouter) {
            return this.#l1SwapRouter
        }
        this.#l1SwapRouter = this.ethereumProvider.connectL1SwapRouter(
            this.l1SwapRouterAddress(),
            this.ethereum(),
        )
        return this.#l1SwapRouter
    }

    l1LegacySwapRouter(): Contract & ISwapLegacyRouter {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1LegacySwapRouter) {
            return this.#l1LegacySwapRouter
        }
        this.#l1LegacySwapRouter = this.ethereumProvider.connectL1LegacySwapRouter(
            this.environment.l2Bridge.l1SwapRouterAddress as string,
            this.ethereum(),
        )
        return this.#l1LegacySwapRouter
    }

    l1HandlerAddress(): string {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        return this.environment.l2Bridge.l1HandlerAddress as string
    }

    l1FeeTokenAddress(): string {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        return this.environment.l2Bridge.l1FeeTokenAddress as string
    }

    l2Adapter(l2ChainId: number): Contract & SnowbridgeL2Adaptor {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l2Adapters[l2ChainId]) {
            return this.#l2Adapters[l2ChainId]
        }
        const adapter = this.ethereumProvider.connectL2Adapter(
            this.environment.l2Bridge.l2Chains[l2ChainId].adapterAddress as string,
            this.ethChain(l2ChainId),
        )
        this.#l2Adapters[l2ChainId] = adapter
        return adapter
    }

    l2FeeTokenAddress(l2ChainId: number): string {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (!this.environment.l2Bridge.l2Chains[l2ChainId]) {
            throw new Error("L2 chain configuration is missing.")
        }
        return this.environment.l2Bridge.l2Chains[l2ChainId].feeTokenAddress as string
    }

    acrossApiUrl(): string {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        return this.environment.l2Bridge.acrossAPIUrl as string
    }
}

export class SnowbridgeApi {
    readonly context: Context
    constructor(options: ApiOptions) {
        this.context = new Context(options.info.environment)
    }

}

type ApiOptions = { info: BridgeInfo }
export function createApi(options: ApiOptions): SnowbridgeApi {
    return new SnowbridgeApi(options)
}