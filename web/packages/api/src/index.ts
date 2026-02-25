// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import {
    BeefyClient,
    BEEFY_CLIENT_ABI,
    IGatewayV1,
    IGATEWAY_V1_ABI,
    IGatewayV2,
    IGATEWAY_V2_ABI,
    ISwapQuoter,
    SNOWBRIDGE_L1_ADAPTOR_ABI,
    SNOWBRIDGE_L2_ADAPTOR_ABI,
    SWAP_LEGACY_ROUTER_ABI,
    SWAP_QUOTER_ABI,
    SWAP_ROUTER_ABI,
} from "./contracts"
import { type EthereumProvider } from "./EthereumProvider"
import { BridgeInfo, ChainId, Environment } from "@snowbridge/base-types"
import { CreateAgent } from "./registration/agent/createAgent"
import type { AgentCreationInterface } from "./registration/agent/agentInterface"

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
export type { EthereumProvider, EthersContext } from "./EthereumProvider"

export class Context<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction,
> {
    readonly environment: Environment
    readonly ethereumProvider: EthereumProvider<
        EConnection,
        EContract,
        EAbi,
        EInterface,
        ETransactionReceipt,
        EContractTransaction
    >

    // Ethereum
    #ethChains: Record<string, EConnection>
    #gateway?: EContract & IGatewayV1
    #gatewayV2?: EContract & IGatewayV2
    #beefyClient?: EContract & BeefyClient
    #l1Adapter?: EContract
    #l1SwapQuoter?: EContract & ISwapQuoter
    #l1SwapRouter?: EContract
    #l1LegacySwapRouter?: EContract
    #l2Adapters: { [l2ChainId: number]: EContract } = {}

    // Substrate
    #polkadotParachains: Record<string, Promise<ApiPromise>>
    #kusamaParachains: Record<string, Promise<ApiPromise>>
    #relaychain?: ApiPromise

    static #rpcInitTimeoutMs = 40_000
    static #wsRequestTimeoutMs = 30_000

    constructor(
        environment: Environment,
        ethereumProvider: EthereumProvider<
            EConnection,
            EContract,
            EAbi,
            EInterface,
            ETransactionReceipt,
            EContractTransaction
        >,
    ) {
        this.environment = environment
        this.ethereumProvider = ethereumProvider
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

    setEthProvider(ethChainId: number, provider: EConnection) {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            this.ethereumProvider.destroyProvider(this.#ethChains[ethChainKey])
        }
        this.#ethChains[ethChainKey] = provider
    }

    ethChain(ethChainId: number): EConnection {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            return this.#ethChains[ethChainKey]
        }

        const { ethereumChains } = this.environment
        if (ethChainKey in ethereumChains) {
            const url = ethereumChains[ethChainKey]
            const provider = this.ethereumProvider.createProvider(url)
            this.#ethChains[ethChainKey] = provider
            return provider
        } else {
            throw Error(`Ethereum chain id ${ethChainKey} not in the list of ethereum urls.`)
        }
    }

    ethereum(): EConnection {
        return this.ethChain(this.environment.ethChainId)
    }

    gateway(): EContract & IGatewayV1 {
        if (this.#gateway) {
            return this.#gateway
        }
        this.#gateway = this.ethereumProvider.connectContract<EContract & IGatewayV1>(
            this.environment.gatewayContract,
            IGATEWAY_V1_ABI as EAbi,
            this.ethereum(),
        )
        return this.#gateway!
    }

    gatewayV2(): EContract & IGatewayV2 {
        if (this.#gatewayV2) {
            return this.#gatewayV2
        }
        this.#gatewayV2 = this.ethereumProvider.connectContract<EContract & IGatewayV2>(
            this.environment.gatewayContract,
            IGATEWAY_V2_ABI as EAbi,
            this.ethereum(),
        )
        return this.#gatewayV2!
    }

    beefyClient(): EContract & BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        this.#beefyClient = this.ethereumProvider.connectContract<EContract & BeefyClient>(
            this.environment.beefyContract,
            BEEFY_CLIENT_ABI as EAbi,
            this.ethereum(),
        )
        return this.#beefyClient!
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

    l1Adapter(): EContract {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1Adapter) {
            return this.#l1Adapter
        }
        this.#l1Adapter = this.ethereumProvider.connectContract<EContract>(
            this.environment.l2Bridge.l1AdapterAddress,
            SNOWBRIDGE_L1_ADAPTOR_ABI as EAbi,
            this.ethereum(),
        )
        return this.#l1Adapter!
    }

    l1SwapQuoter(): EContract {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1SwapQuoter) {
            return this.#l1SwapQuoter
        }
        this.#l1SwapQuoter = this.ethereumProvider.connectContract<EContract & ISwapQuoter>(
            this.environment.l2Bridge.l1SwapQuoterAddress,
            SWAP_QUOTER_ABI as EAbi,
            this.ethereum(),
        )
        return this.#l1SwapQuoter!
    }

    l1SwapRouter(): EContract {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1SwapRouter) {
            return this.#l1SwapRouter
        }
        this.#l1SwapRouter = this.ethereumProvider.connectContract<EContract>(
            this.environment.l2Bridge.l1SwapRouterAddress,
            SWAP_ROUTER_ABI as EAbi,
            this.ethereum(),
        )
        return this.#l1SwapRouter!
    }

    l1LegacySwapRouter(): EContract {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1LegacySwapRouter) {
            return this.#l1LegacySwapRouter
        }
        this.#l1LegacySwapRouter = this.ethereumProvider.connectContract<EContract>(
            this.environment.l2Bridge.l1SwapRouterAddress,
            SWAP_LEGACY_ROUTER_ABI as EAbi,
            this.ethereum(),
        )
        return this.#l1LegacySwapRouter!
    }

    l2Adapter(l2ChainId: number): EContract {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l2Adapters[l2ChainId]) {
            return this.#l2Adapters[l2ChainId]
        }
        const adapter = this.ethereumProvider.connectContract<EContract>(
            this.environment.l2Bridge.l2Chains[l2ChainId].adapterAddress,
            SNOWBRIDGE_L2_ADAPTOR_ABI as EAbi,
            this.ethChain(l2ChainId),
        )
        this.#l2Adapters[l2ChainId] = adapter
        return adapter
    }
}

export type ApiOptions<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction,
> = {
    info: BridgeInfo
    ethereumProvider: EthereumProvider<
        EConnection,
        EContract,
        EAbi,
        EInterface,
        ETransactionReceipt,
        EContractTransaction
    >
}

export class SnowbridgeApi<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction,
> {
    readonly info: BridgeInfo
    readonly context: Context<
        EConnection,
        EContract,
        EAbi,
        EInterface,
        ETransactionReceipt,
        EContractTransaction
    >
    constructor(
        options: ApiOptions<
            EConnection,
            EContract,
            EAbi,
            EInterface,
            ETransactionReceipt,
            EContractTransaction
        >,
    ) {
        this.info = options.info
        this.context = new Context(options.info.environment, options.ethereumProvider)
    }
    createAgent(): AgentCreationInterface<
        Context<
            EConnection,
            EContract,
            EAbi,
            EInterface,
            ETransactionReceipt,
            EContractTransaction
        >,
        EContractTransaction
    > {
        return new CreateAgent(this.context, this.info.registry)
    }
    transfer(
        from: ChainId,
        to: ChainId,
    ) {

    }
}

export function createApi<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction,
>(
    options: ApiOptions<
        EConnection,
        EContract,
        EAbi,
        EInterface,
        ETransactionReceipt,
        EContractTransaction
    >,
): SnowbridgeApi<
    EConnection,
    EContract,
    EAbi,
    EInterface,
    ETransactionReceipt,
    EContractTransaction
> {
    return new SnowbridgeApi(options)
}
