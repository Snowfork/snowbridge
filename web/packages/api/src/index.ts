// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { AbstractProvider, JsonRpcProvider, WebSocketProvider } from "ethers"
import {
    BeefyClient,
    BeefyClient__factory,
    IGateway,
    IGateway__factory,
} from "@snowbridge/contract-types"

interface Parachains { [paraId: string]: ApiPromise }

interface Config {
    ethereum: {
        execution_url: string | AbstractProvider
        beacon_url: string
    }
    polkadot: {
        relaychain: string
        assetHubParaId: number
        bridgeHubParaId: number
        parachains: { [paraId: string]: string }
    }
    appContracts: {
        gateway: string
        beefy: string
    }
    graphqlApiUrl?: string
}

export class Context {
    config: Config

    // Ethereum
    #ethereum?: AbstractProvider
    #gateway?: IGateway
    #beefyClient?: BeefyClient

    // Substrate
    #parachains: Parachains
    #relaychain?: ApiPromise

    constructor(config: Config) {
        this.config = config
        this.#parachains = {}
    }

    async relaychain(): Promise<ApiPromise> {
        if (this.#relaychain) {
            return this.#relaychain
        }
        const url = this.config.polkadot.relaychain
        this.#relaychain = await ApiPromise.create({
            provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
        })
        return this.#relaychain
    }

    assetHub(): Promise<ApiPromise> {
        return this.parachain(this.config.polkadot.assetHubParaId)
    }

    bridgeHub(): Promise<ApiPromise> {
        return this.parachain(this.config.polkadot.bridgeHubParaId)
    }

    hasParachain(paraId: number): boolean {
        return paraId.toString() in this.config.polkadot.parachains
    }

    parachains(): number[] {
        return Object.keys(this.config.polkadot.parachains).map((key) => Number(key))
    }

    async parachain(paraId: number): Promise<ApiPromise> {
        const paraIdKey = paraId.toString()
        if (paraIdKey in this.#parachains) {
            return this.#parachains[paraIdKey]
        }
        const { parachains } = this.config.polkadot
        if (paraIdKey in parachains) {
            const url = parachains[paraIdKey]
            const api = await ApiPromise.create({
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
            })
            const onChainParaId = (
                await api.query.parachainInfo.parachainId()
            ).toPrimitive() as number
            if(onChainParaId !== paraId) {
                console.warn(
                    `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}, url=${url}`
                )
            }
            this.#parachains[onChainParaId] = api
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
            await this.#relaychain.disconnect()
        }

        for (const paraId of Object.keys(this.#parachains)) {
            await this.#parachains[Number(paraId)].disconnect()
        }
    }
}

export * as toPolkadot from "./toPolkadot"
export * as toPolkadotV2 from "./toPolkadot_v2"
export * as toEthereum from "./toEthereum"
export * as utils from "./utils"
export * as status from "./status"
export * as assets from "./assets"
export * as assetsV2 from "./assets_v2"
export * as environment from "./environment"
export * as subscan from "./subscan"
export * as history from "./history"
export * as historyV2 from "./history_v2"
export * as subsquid from "./subsquid"
