import type { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import {
    BeefyClient,
    BEEFY_CLIENT_ABI,
    BridgeInfo,
    IGatewayV1,
    IGATEWAY_V1_ABI,
    IGatewayV2,
    IGATEWAY_V2_ABI,
    ISwapQuoter,
    SWAP_QUOTER_ABI,
    ChainId,
    Environment,
    EthereumChain,
    EthereumProvider,
    EthereumProviderTypes,
    Parachain,
    TransferKind as BaseTransferKind,
} from "@snowbridge/base-types"
import type { AddTipInterface } from "./addTip/addTipInterface"
import type { AgentCreationInterface } from "./types/registration/agent"
import type { RegistrationInterface } from "./types/registration/toPolkadot"
import type { TransferInterface as ForInterParachainTransferInterface } from "./transfers/forInterParachain/transferInterface"
import type { TransferInterface as ForKusamaTransferInterface } from "./transfers/forKusama/transferInterface"
import type { TransferInterface as ToPolkadotTransferInterface } from "./transfers/toPolkadot/transferInterface"
import type { TransferInterface as ToPolkadotL2TransferInterface } from "./transfers/l2ToPolkadot/transferInterface"
import type { TransferInterface as ToEthereumTransferInterface } from "./transfers/toEthereum/transferInterface"
import type { TransferInterface as ToEthereumL2TransferInterface } from "./transfers/polkadotToL2/transferInterface"
import type { TransferInterface as ToEthereumEvmTransferInterface } from "./transfers/toEthereumEvm/transferInterface"
import { toEthereumTransferById, toPolkadotTransferById } from "./history_v2"
import type { ToEthereumTransferResult, ToPolkadotTransferResult } from "./history"

export type {
    MessageDirection,
    TipAddition,
    TipAdditionParams,
    TipAdditionResponse,
    TipAdditionValidationLog,
    TipAsset,
    ValidatedTipAddition,
} from "./types/addTip"
export type { AddTipInterface } from "./addTip/addTipInterface"
export { TipAdditionValidationKind } from "./types/addTip"
export type {
    AgentCreation,
    AgentCreationInterface,
    ValidatedCreateAgent,
} from "./types/registration/agent"
export type {
    RegistrationFee,
    RegistrationInterface,
    TokenRegistration,
    ValidatedRegisterToken,
} from "./types/registration/toPolkadot"
export * as toPolkadotV2 from "./types/toPolkadot"
export * as toPolkadotSnowbridgeV2 from "./types/toPolkadotSnowbridgeV2"
export * as toEthereumV2 from "./types/toEthereum"
export * as toEthereumFromEVMV2 from "./types/toEthereumEvm"
export * as forInterParachain from "./types/forInterParachain"
export * as forKusama from "./types/forKusama"
export * as feeSchedule from "./feeSchedule"
export type { VolumeFeeParams } from "./feeSchedule"
export type {
    FeeAsset,
    FeeItem,
    ToPolkadotFeeKey,
    L2ToPolkadotFeeKey,
    ToEthereumFeeKey,
    InterParachainFeeKey,
    KusamaFeeKey,
    V1ToPolkadotFeeKey,
} from "./types/fee"
export { addBreakdown, computeTotals, findInBreakdown, findTotal } from "./fees"
export * as utils from "./utils"
export * as status from "./status"
export * as assetsV2 from "./assets_v2"
export * as history from "./history"
export * as historyV2 from "./history_v2"
export { TransferStatus } from "./history_v2"
export * as subsquidV2 from "./subsquid_v2"

export class Context<T extends EthereumProviderTypes> {
    readonly environment: Environment
    readonly ethereumProvider: EthereumProvider<T>

    // Ethereum
    #ethChains: Record<string, T["Connection"]>
    #gateway?: T["Contract"] & IGatewayV1
    #gatewayV2?: T["Contract"] & IGatewayV2
    #beefyClient?: T["Contract"] & BeefyClient
    #l1SwapQuoter?: T["Contract"] & ISwapQuoter

    // Substrate
    #polkadotParachains: Record<string, Promise<ApiPromise>>
    #kusamaParachains: Record<string, Promise<ApiPromise>>
    #relaychain?: ApiPromise

    static #rpcInitTimeoutMs = 40_000
    static #wsRequestTimeoutMs = 30_000

    constructor(environment: Environment, ethereumProvider: EthereumProvider<T>) {
        this.environment = environment
        this.ethereumProvider = ethereumProvider
        this.#polkadotParachains = {}
        this.#kusamaParachains = {}
        this.#ethChains = {}
    }

    #polkadotApi() {
        return require("@polkadot/api") as typeof import("@polkadot/api")
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
            const { ApiPromise } = this.#polkadotApi()
            return await Promise.race([ApiPromise.create(options), timeoutPromise])
        } finally {
            if (timer) clearTimeout(timer)
        }
    }

    #buildProvider(url: string) {
        const { HttpProvider, WsProvider } = this.#polkadotApi()
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

    async paraImplementation(provider: ApiPromise) {
        const { paraImplementation } = require("./parachains") as {
            paraImplementation: typeof import("./parachains").paraImplementation
        }
        return paraImplementation(provider, this.ethereumProvider)
    }

    setEthProvider(ethChainId: number, provider: T["Connection"]) {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            this.ethereumProvider.destroyProvider(this.#ethChains[ethChainKey])
        }
        this.#ethChains[ethChainKey] = provider
    }

    ethChain(ethChainId: number): T["Connection"] {
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

    ethereum(): T["Connection"] {
        return this.ethChain(this.environment.ethChainId)
    }

    gateway(): T["Contract"] & IGatewayV1 {
        if (this.#gateway) {
            return this.#gateway
        }
        this.#gateway = this.ethereumProvider.connectContract(
            this.environment.gatewayContract,
            IGATEWAY_V1_ABI as T["Abi"],
            this.ethereum(),
        ) as T["Contract"] & IGatewayV1
        return this.#gateway!
    }

    gatewayV2(): T["Contract"] & IGatewayV2 {
        if (this.#gatewayV2) {
            return this.#gatewayV2
        }
        this.#gatewayV2 = this.ethereumProvider.connectContract(
            this.environment.gatewayContract,
            IGATEWAY_V2_ABI as T["Abi"],
            this.ethereum(),
        ) as T["Contract"] & IGatewayV2
        return this.#gatewayV2!
    }

    beefyClient(): T["Contract"] & BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        this.#beefyClient = this.ethereumProvider.connectContract(
            this.environment.beefyContract,
            BEEFY_CLIENT_ABI as T["Abi"],
            this.ethereum(),
        ) as T["Contract"] & BeefyClient
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

    l1SwapQuoter(): T["Contract"] {
        if (!this.environment.l2Bridge) {
            throw new Error("L2 bridge configuration is missing.")
        }
        if (this.#l1SwapQuoter) {
            return this.#l1SwapQuoter
        }
        this.#l1SwapQuoter = this.ethereumProvider.connectContract(
            this.environment.l2Bridge.l1SwapQuoterAddress,
            SWAP_QUOTER_ABI as T["Abi"],
            this.ethereum(),
        ) as T["Contract"] & ISwapQuoter
        return this.#l1SwapQuoter!
    }
}

export type ApiOptions<P extends EthereumProvider<any>> = {
    info: BridgeInfo
    ethereumProvider: P
}

export type TransferImplementation<T extends EthereumProviderTypes = EthereumProviderTypes> =
    | ({ kind: "polkadot->polkadot" } & ForInterParachainTransferInterface<T>)
    | ({ kind: "kusama->polkadot" } & ForKusamaTransferInterface<T>)
    | ({ kind: "polkadot->kusama" } & ForKusamaTransferInterface<T>)
    | ({ kind: "polkadot->ethereum" } & ToEthereumTransferInterface<T>)
    | ({ kind: "ethereum->polkadot" } & ToPolkadotTransferInterface<T>)
    | ({ kind: "ethereum->ethereum" } & ToEthereumEvmTransferInterface<T>)
    | ({ kind: "polkadot->ethereum_l2" } & ToEthereumL2TransferInterface<T>)
    | ({ kind: "ethereum_l2->polkadot" } & ToPolkadotL2TransferInterface<T>)

type TransferKindFor<T extends EthereumProviderTypes> = Extract<
    TransferImplementation<T>["kind"],
    BaseTransferKind
>
type TransferForKind<K extends TransferKindFor<T>, T extends EthereumProviderTypes> = Extract<
    TransferImplementation<T>,
    { kind: K }
>
type TransferFromTo<
    F extends ChainId,
    To extends ChainId,
    T extends EthereumProviderTypes,
> = TransferForKind<Extract<`${F["kind"]}->${To["kind"]}`, TransferKindFor<T>>, T>
type ProviderTypesFor<P extends EthereumProvider<any>> = P["providerTypes"]

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

export class SnowbridgeApi<P extends EthereumProvider<any>> {
    readonly info: BridgeInfo
    readonly context: Context<ProviderTypesFor<P>>
    constructor(options: ApiOptions<P>) {
        this.info = options.info
        this.context = new Context<ProviderTypesFor<P>>(
            options.info.environment,
            options.ethereumProvider as unknown as EthereumProvider<ProviderTypesFor<P>>,
        )
    }
    createAgent(): AgentCreationInterface<ProviderTypesFor<P>["ContractTransaction"]> {
        const { CreateAgent } = require("./registration/agent/createAgent") as {
            CreateAgent: new (
                context: Context<ProviderTypesFor<P>>,
                registry: BridgeInfo["registry"],
            ) => AgentCreationInterface<ProviderTypesFor<P>["ContractTransaction"]>
        }
        return new CreateAgent(this.context, this.info.registry)
    }
    registerToken(): RegistrationInterface<ProviderTypesFor<P>> {
        const { RegisterToken } = require("./registration/toPolkadot/registerToken") as {
            RegisterToken: new (
                context: Context<ProviderTypesFor<P>>,
                registry: BridgeInfo["registry"],
            ) => RegistrationInterface<ProviderTypesFor<P>>
        }
        return new RegisterToken(this.context, this.info.registry)
    }
    addTip(): AddTipInterface<ProviderTypesFor<P>> {
        const { AddTip } = require("./addTip/addTip") as {
            AddTip: new (
                context: Context<ProviderTypesFor<P>>,
                registry: BridgeInfo["registry"],
            ) => AddTipInterface<ProviderTypesFor<P>>
        }
        return new AddTip(this.context, this.info.registry)
    }
    sender<F extends ChainId, T extends ChainId>(
        from: F,
        to: T,
    ): TransferFromTo<F, T, ProviderTypesFor<P>>
    sender<F extends ChainId, T extends ChainId>(
        from: F,
        to: T,
    ): TransferFromTo<F, T, ProviderTypesFor<P>> {
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
                const { InterParachainTransfer } = require("./forInterParachain")
                return withKind(
                    new InterParachainTransfer(
                        this.info,
                        this.context,
                        route,
                        sourceParachain,
                        destinationParachain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "kusama->polkadot":
            case "polkadot->kusama": {
                const sourceParachain = sourceChain as Parachain
                const destinationParachain = destinationChain as Parachain
                const { KusamaTransfer } = require("./forKusama")
                return withKind(
                    new KusamaTransfer(
                        this.info,
                        this.context,
                        route,
                        sourceParachain,
                        destinationParachain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "polkadot->ethereum": {
                const sourceParachain = sourceChain as Parachain
                const destinationEthChain = destinationChain as EthereumChain
                if (sourceParachain.features.supportsV2) {
                    const { TransferToEthereum } = require("./toEthereumSnowbridgeV2")
                    return withKind(
                        new TransferToEthereum(
                            this.context,
                            route,
                            this.info.registry,
                            sourceParachain,
                            destinationEthChain,
                        ),
                        kind,
                    ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
                }
                const { V1ToEthereumAdapter } = require("./toEthereum_v2")
                return withKind(
                    new V1ToEthereumAdapter(
                        this.context,
                        this.info.registry,
                        route,
                        sourceParachain,
                        destinationEthChain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "ethereum->polkadot": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationParachain = destinationChain as Parachain
                if (destinationParachain.features.supportsV2) {
                    const { TransferToPolkadot } = require("./toPolkadotSnowbridgeV2")
                    return withKind(
                        new TransferToPolkadot(
                            this.context,
                            route,
                            this.info.registry,
                            sourceEthChain,
                            destinationParachain,
                        ),
                        kind,
                    ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
                }
                const { V1ToPolkadotAdapter } = require("./toPolkadot_v2")
                return withKind(
                    new V1ToPolkadotAdapter(
                        this.context,
                        this.info.registry,
                        route,
                        sourceEthChain,
                        destinationParachain,
                    ),
                    kind,
                ) as unknown as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "ethereum->ethereum": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationEthChain = destinationChain as EthereumChain
                const { V1ToEthereumEvmAdapter } = require("./toEthereumFromEVM_v2")
                const tIface: ToEthereumEvmTransferInterface<ProviderTypesFor<P>> =
                    new V1ToEthereumEvmAdapter(
                        this.context,
                        this.info.registry,
                        route,
                        sourceEthChain,
                        destinationEthChain,
                    )
                return withKind(tIface, kind) as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "polkadot->ethereum_l2": {
                const sourceParachain = sourceChain as Parachain
                const destinationEthChain = destinationChain as EthereumChain
                const { ERC20FromAH } = require("./transfers/polkadotToL2/erc20ToL2")
                const tIface: ToEthereumL2TransferInterface<ProviderTypesFor<P>> = new ERC20FromAH(
                    this.context,
                    this.info.registry,
                    route,
                    sourceParachain,
                    destinationEthChain,
                )
                return withKind(tIface, kind) as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            case "ethereum_l2->polkadot": {
                const sourceEthChain = sourceChain as EthereumChain
                const destinationParachain = destinationChain as Parachain
                const { ERC20ToAH } = require("./transfers/l2ToPolkadot/erc20ToAH")
                const tIface: ToPolkadotL2TransferInterface<ProviderTypesFor<P>> = new ERC20ToAH(
                    this.context,
                    this.info.registry,
                    route,
                    sourceEthChain,
                    destinationParachain,
                )
                return withKind(tIface, kind) as TransferFromTo<F, T, ProviderTypesFor<P>>
            }
            default:
                throw new Error(`No implementation for route ${route.from.kind}:${route.to.kind}.`)
        }
    }

    async destroy(): Promise<void> {
        await this.context.destroyContext()
    }

    async txStatus(
        messageId: string,
    ): Promise<ToPolkadotTransferResult | ToEthereumTransferResult | undefined> {
        const graphqlApiUrl = this.context.graphqlApiUrl()
        const [toPolkadot, toEthereum] = await Promise.all([
            toPolkadotTransferById(graphqlApiUrl, messageId),
            toEthereumTransferById(graphqlApiUrl, messageId),
        ])
        return toPolkadot ?? toEthereum
    }
}

export function createApi<P extends EthereumProvider<any>>(
    options: ApiOptions<P>,
): SnowbridgeApi<P> {
    return new SnowbridgeApi(options)
}
