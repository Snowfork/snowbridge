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
        assetHubParaId: number
        bridgeHubParaId: number
        relaychain: string
        parachains: { [paraId: number]: string }
    }
    appContracts: {
        gateway: string
        beefy: string
    }
    graphqlApiUrl?: string
}

export type ChainProperties = {
    tokenSymbol: string
    tokenDecimal: number
    ss58Format: number
    isEthereum: boolean
}

export interface SusbtrateChain {
    get api(): ApiPromise
    properties(): Promise<ChainProperties>
}

export interface Parachain extends SusbtrateChain {
    get parachainId(): number
    get isSystemParachain(): boolean
}

export class SubstrateChain implements SusbtrateChain {
    #api: ApiPromise
    #properties?: ChainProperties

    constructor(api: ApiPromise) {
        this.#api = api
    }
    async properties(): Promise<ChainProperties> {
        if (this.#properties) {
            return this.#properties
        }
        const properties = await this.#api.rpc.system.properties()
        const tokenSymbols = properties.tokenSymbol.unwrapOrDefault()
        const tokenDecimals = properties.tokenDecimals.unwrapOrDefault()
        const isEthereum = properties.isEthereum.toPrimitive()

        return {
            tokenSymbol: tokenSymbols.at(0)?.toString() ?? "DOT",
            tokenDecimal: tokenDecimals.at(0)?.toNumber() ?? 10,
            ss58Format: properties.ss58Format.unwrapOr(null)?.toNumber() ?? 42,
            isEthereum: isEthereum,
        }
    }

    get api(): ApiPromise {
        return this.#api
    }
}

export class GenericParachain extends SubstrateChain implements Parachain {
    #parachainId: number

    constructor(parachainId: number, api: ApiPromise) {
        super(api)
        this.#parachainId = parachainId
    }
    get isSystemParachain(): boolean {
        return this.#parachainId < 2000
    }

    get parachainId(): number {
        return this.#parachainId
    }
}

export class Context {
    config: Config

    // Ethereum
    #ethereum?: AbstractProvider
    #gateway?: IGateway
    #beefyClient?: BeefyClient

    // Substrate
    #parachains: { [paraId: number]: GenericParachain }
    #relaychain?: SubstrateChain

    constructor(config: Config) {
        this.config = config
        this.#parachains = {}
    }

    async relaychain(): Promise<SubstrateChain> {
        if (this.#relaychain) {
            return this.#relaychain
        }
        const url = this.config.polkadot.relaychain
        const api = await ApiPromise.create({
            provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
        })
        this.#relaychain = new SubstrateChain(api)
        return this.#relaychain
    }

    async assetHub(): Promise<GenericParachain> {
        return this.parachain(this.config.polkadot.assetHubParaId)
    }

    bridgeHub(): Promise<GenericParachain> {
        return this.parachain(this.config.polkadot.bridgeHubParaId)
    }

    hasParachain(paraId: number): boolean {
        return paraId in this.config.polkadot.parachains
    }

    parachains(): number[] {
        return Object.keys(this.config.polkadot.parachains).map((key) => Number(key))
    }

    async parachain(paraId: number): Promise<GenericParachain> {
        if (paraId in this.#parachains) {
            return this.#parachains[paraId]
        }
        const { parachains } = this.config.polkadot
        if (paraId in parachains) {
            const url = parachains[paraId]
            const api = await ApiPromise.create({
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
            })
            const onChainParaId = (
                await api.query.parachainInfo.parachainId()
            ).toPrimitive() as number
            console.warn(
                `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}`
            )
            this.#parachains[onChainParaId] = new GenericParachain(onChainParaId, api)
            return this.#parachains[onChainParaId]
        } else {
            throw Error(`Parachain id ${paraId} not in the list of parachain urls.`)
        }
    }

    ethereum(): AbstractProvider {
        if (this.#ethereum) {
            return this.#ethereum
        }

        const { config } = this

        if (typeof config.ethereum.execution_url === "string") {
            if (config.ethereum.execution_url.startsWith("http")) {
                this.#ethereum = new JsonRpcProvider(config.ethereum.execution_url)
            } else {
                this.#ethereum = new WebSocketProvider(config.ethereum.execution_url)
            }
        } else {
            this.#ethereum = this.config.ethereum.execution_url as AbstractProvider
        }

        return this.#ethereum
    }

    gateway(): IGateway {
        if (this.#gateway) {
            return this.#gateway
        }
        return IGateway__factory.connect(this.config.appContracts.gateway, this.ethereum())
    }
    beefyClient(): BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        return BeefyClient__factory.connect(this.config.appContracts.beefy, this.ethereum())
    }

    async destroyContext(): Promise<void> {
        // clean up etheruem
        if (typeof this.config.ethereum.execution_url === "string" && this.#ethereum) {
            await this.beefyClient().removeAllListeners()
            await this.gateway().removeAllListeners()
            this.ethereum().destroy()
        }
        // clean up polkadot
        if (this.#relaychain) {
            await this.#relaychain.api.disconnect()
        }

        for (const paraId of Object.keys(this.#parachains)) {
            await this.#parachains[Number(paraId)].api.disconnect()
        }
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
