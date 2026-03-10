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
    SNOWBRIDGE_L2_ADAPTOR_ABI,
    SWAP_QUOTER_ABI,
} from "./contracts"
import { type EthereumProvider } from "./EthereumProvider"
import {
    BridgeInfo,
    ChainId,
    Environment,
    EthereumChain,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import { CreateAgent } from "./registration/agent/createAgent"
import type { AgentCreationInterface } from "./registration/agent/agentInterface"
import * as kusamaTransfers from "./forKusama"
import * as interParachainTransfers from "./forInterParachain"
import * as toEthereumTransfers from "./toEthereumSnowbridgeV2"
import * as toPolkadotTransfers from "./toPolkadotSnowbridgeV2"
import * as toEthereumEvmTransfers from "./toEthereumFromEVM_v2"
import * as toEthereumTransfersV1 from "./toEthereum_v2"
import * as toPolkadotTransfersV1 from "./toPolkadot_v2"
import type { TransferInterface as ForInterParachainTransferInterface } from "./transfers/forInterParachain/transferInterface"
import type { TransferInterface as ForKusamaTransferInterface } from "./transfers/forKusama/transferInterface"
import type { TransferInterface as ToPolkadotTransferInterface } from "./transfers/toPolkadot/transferInterface"
import type { TransferInterface as ToPolkadotL2TransferInterface } from "./transfers/l2ToPolkadot/transferInterface"
import type { TransferInterface as ToEthereumTransferInterface } from "./transfers/toEthereum/transferInterface"
import type { TransferInterface as ToEthereumL2TransferInterface } from "./transfers/polkadotToL2/transferInterface"
import type { TransferInterface as ToEthereumEvmTransferInterface } from "./transfers/toEthereumEvm/transferInterface"
import { ERC20ToAH as ERC20FromL2ToAH } from "./transfers/l2ToPolkadot/erc20ToAH"
import { ERC20FromAH as ERC20FromAHToL2 } from "./transfers/polkadotToL2/erc20ToL2"
import { V1ToEthereumEvmAdapter } from "./toEthereumFromEVM_v2"

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
    #l1SwapQuoter?: EContract & ISwapQuoter
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

export type TransferImplementation =
    | ({ kind: "polkadot->polkadot" } & ForInterParachainTransferInterface)
    | ({ kind: "kusama->polkadot" } & ForKusamaTransferInterface)
    | ({ kind: "polkadot->kusama" } & ForKusamaTransferInterface)
    | ({ kind: "polkadot->ethereum" } & ToEthereumTransferInterface)
    | ({ kind: "ethereum->polkadot" } & ToPolkadotTransferInterface)
    | ({ kind: "ethereum->ethereum" } & ToEthereumEvmTransferInterface)
    | ({ kind: "polkadot->ethereum_l2" } & ToEthereumL2TransferInterface)
    | ({ kind: "ethereum_l2->polkadot" } & ToPolkadotL2TransferInterface)

type TransferKind = TransferImplementation["kind"]
type TransferForKind<K extends TransferKind> = Extract<TransferImplementation, { kind: K }>
type TransferFromTo<F extends ChainId, T extends ChainId> = TransferForKind<
    Extract<`${F["kind"]}->${T["kind"]}`, TransferKind>
>

function withKind<K extends TransferImplementation["kind"], T>(
    implementation: T,
    kind: K,
): T & { kind: K } {
    return Object.assign(implementation as object, { kind }) as T & { kind: K }
}

type RegistryTransferChain = EthereumChain | Parachain

function resolveRegistryTransferChain(info: BridgeInfo, chain: ChainId): RegistryTransferChain {
    switch (chain.kind) {
        case "polkadot": {
            const parachain = info.registry.parachains[`polkadot_${chain.id}`]
            if (!parachain) {
                throw new Error(`Could not find polkadot parachain ${chain.id} in the registry.`)
            }
            return parachain
        }
        case "kusama": {
            const parachain = info.registry.kusama?.parachains[`kusama_${chain.id}`]
            if (!parachain) {
                throw new Error(`Could not find kusama parachain ${chain.id} in the registry.`)
            }
            return parachain
        }
        case "ethereum":
        case "ethereum_l2": {
            const ethChain = info.registry.ethereumChains[`${chain.kind}_${chain.id}`]
            if (!ethChain) {
                throw new Error(`Could not find ${chain.kind} chain ${chain.id} in the registry.`)
            }
            return ethChain
        }
    }
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
    transfer<F extends ChainId, T extends ChainId>(from: F, to: T): TransferFromTo<F, T>
    transfer<F extends ChainId, T extends ChainId>(from: F, to: T): TransferFromTo<F, T> {
        const source = from
        const destination = to

        const route = this.info.routes.find(
            (entry) =>
                entry.from.kind === source.kind &&
                entry.from.id === source.id &&
                entry.to.kind === destination.kind &&
                entry.to.id === destination.id,
        )
        if (!route) {
            throw new Error(
                `No route for ${source.kind}:${source.id} -> ${destination.kind}:${destination.id}.`,
            )
        }

        const kind = `${route.from.kind}->${route.to.kind}` as const
        const sourceChain = resolveRegistryTransferChain(this.info, source)
        const destinationChain = resolveRegistryTransferChain(this.info, destination)

        switch (kind) {
            case "polkadot->polkadot": {
                const sourceParachain = sourceChain as Parachain
                const destinationParachain = destinationChain as Parachain
                return withKind(
                    new interParachainTransfers.InterParachainTransfer(
                        this.info,
                        this.context as any,
                        route,
                        sourceParachain,
                        destinationParachain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T>
            }
            case "kusama->polkadot":
            case "polkadot->kusama": {
                const sourceParachain = sourceChain as Parachain
                const destinationParachain = destinationChain as Parachain
                return withKind(
                    new kusamaTransfers.KusamaTransfer(
                        this.info,
                        this.context as any,
                        route,
                        sourceParachain,
                        destinationParachain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T>
            }
            case "polkadot->ethereum": {
                const sourceParachain = sourceChain as Parachain
                const destinationEthChain = destinationChain as EthereumChain
                return withKind(
                    sourceParachain.features.supportsV2
                        ? toEthereumTransfers.createTransferImplementation(
                              this.context as any,
                              route,
                              this.info.registry,
                              sourceParachain,
                              destinationEthChain,
                          )
                        : toEthereumTransfersV1.createTransferImplementationV1(
                              this.context as any,
                              route,
                              this.info.registry,
                              sourceParachain,
                              destinationEthChain,
                          ),
                    kind,
                ) as TransferFromTo<F, T>
            }
            case "ethereum->polkadot": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationParachain = destinationChain as Parachain
                return withKind(
                    destinationParachain.features.supportsV2
                        ? toPolkadotTransfers.createTransferImplementation(
                              this.context as any,
                              route,
                              this.info.registry,
                              sourceEthChain,
                              destinationParachain,
                          )
                        : toPolkadotTransfersV1.createTransferImplementationV1(
                              this.context as any,
                              route,
                              this.info.registry,
                              sourceEthChain,
                              destinationParachain,
                          ),
                    kind,
                ) as TransferFromTo<F, T>
            }
            case "ethereum->ethereum": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationEthChain = destinationChain as EthereumChain
                const tIface: ToEthereumEvmTransferInterface = new V1ToEthereumEvmAdapter(
                    this.context as any,
                    this.info.registry,
                    route,
                    sourceEthChain,
                    destinationEthChain,
                )
                return withKind(tIface, kind) as TransferFromTo<F, T>
            }
            case "polkadot->ethereum_l2": {
                const sourceParachain = sourceChain as Parachain
                const destinationEthChain = destinationChain as EthereumChain
                const tIface: ToEthereumL2TransferInterface = new ERC20FromAHToL2(
                    this.context as any,
                    this.info.registry,
                    route,
                    sourceParachain,
                    destinationEthChain,
                )
                return withKind(tIface, kind) as TransferFromTo<F, T>
            }
            case "ethereum_l2->polkadot": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationParachain = destinationChain as Parachain
                const tIface: ToPolkadotL2TransferInterface = new ERC20FromL2ToAH(
                    this.context as any,
                    this.info.registry,
                    route,
                    sourceEthChain,
                    destinationParachain,
                )
                return withKind(tIface, kind) as TransferFromTo<F, T>
            }
            default:
                throw new Error(`No implementation for route ${route.from.kind}:${route.to.kind}.`)
        }
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
