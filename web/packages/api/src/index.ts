// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { AbstractProvider, JsonRpcProvider, WebSocketProvider } from "ethers"
import {
    BeefyClient,
    BeefyClient__factory,
    IGatewayV1 as IGateway,
    IGatewayV1__factory as IGateway__factory,
} from "@snowbridge/contract-types"

interface Parachains {
    [paraId: string]: ApiPromise
}
interface EthereumChains {
    [ethChainId: string]: AbstractProvider
}

interface Config {
    environment: string
    ethereum: {
        ethChainId: number
        ethChains: { [ethChainId: string]: string | AbstractProvider }
        beacon_url: string
    }
    polkadot: {
        relaychain: string
        assetHubParaId: number
        bridgeHubParaId: number
        parachains: { [paraId: string]: string }
    }
    kusama?: {
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
    #ethChains: EthereumChains
    #gateway?: IGateway
    #beefyClient?: BeefyClient

    // Substrate
    #polkadotParachains: Parachains
    #kusamaParachains: Parachains
    #relaychain?: ApiPromise

    constructor(config: Config) {
        this.config = config
        this.#polkadotParachains = {}
        this.#kusamaParachains = {}
        this.#ethChains = {}
    }

    async relaychain(): Promise<ApiPromise> {
        if (this.#relaychain) {
            return this.#relaychain
        }
        const url = this.config.polkadot.relaychain
        console.log("Connecting to the relaychain.")
        this.#relaychain = await ApiPromise.create({
            noInitWarn: true,
            provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
        })
        console.log("Connected to the relaychain.")
        return this.#relaychain
    }

    assetHub(): Promise<ApiPromise> {
        return this.parachain(this.config.polkadot.assetHubParaId)
    }

    kusamaAssetHub(): Promise<ApiPromise> | undefined {
        const assetHubParaId = this.config.kusama?.assetHubParaId
        if (assetHubParaId) {
            return this.kusamaParachain(assetHubParaId)
        }
    }

    bridgeHub(): Promise<ApiPromise> {
        return this.parachain(this.config.polkadot.bridgeHubParaId)
    }

    hasParachain(paraId: number): boolean {
        return paraId.toString() in this.config.polkadot.parachains
    }

    hasEthChain(ethChainId: number): boolean {
        return ethChainId.toString() in this.config.ethereum.ethChains
    }

    parachains(): number[] {
        return Object.keys(this.config.polkadot.parachains).map((key) => Number(key))
    }

    ethChains(): number[] {
        return Object.keys(this.config.ethereum.ethChains).map((key) => Number(key))
    }

    async parachain(paraId: number): Promise<ApiPromise> {
        const paraIdKey = paraId.toString()
        if (paraIdKey in this.#polkadotParachains) {
            return this.#polkadotParachains[paraIdKey]
        }
        const { parachains } = this.config.polkadot
        if (paraIdKey in parachains) {
            const url = parachains[paraIdKey]
            console.log("Connecting to parachain ", paraIdKey, url)
            const api = await ApiPromise.create({
                noInitWarn: true,
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
            })
            const onChainParaId = (
                await api.query.parachainInfo.parachainId()
            ).toPrimitive() as number
            if (onChainParaId !== paraId) {
                console.warn(
                    `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}, url=${url}`
                )
            }
            this.#polkadotParachains[onChainParaId] = api
            console.log("Connected to parachain ", paraIdKey)
            return this.#polkadotParachains[onChainParaId]
        } else {
            throw Error(`Parachain id ${paraId} not in the list of parachain urls.`)
        }
    }

    async kusamaParachain(paraId: number): Promise<ApiPromise> {
        const paraIdKey = paraId.toString()
        if (paraIdKey in this.#kusamaParachains) {
            return this.#kusamaParachains[paraIdKey]
        }
        if (!this.config.kusama) {
            throw Error(`Kusama config is not set.`)
        }
        const { parachains } = this.config.kusama
        if (paraIdKey in parachains) {
            const url = parachains[paraIdKey]
            console.log("Connecting to Kusama parachain ", paraIdKey, url)
            const api = await ApiPromise.create({
                noInitWarn: true,
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
            })
            const onChainParaId = (
                await api.query.parachainInfo.parachainId()
            ).toPrimitive() as number
            if (onChainParaId !== paraId) {
                console.warn(
                    `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}, url=${url}`
                )
            }
            this.#kusamaParachains[onChainParaId] = api
            console.log("Connected to Kusama parachain ", paraIdKey)
            return this.#kusamaParachains[onChainParaId]
        } else {
            throw Error(`Parachain id ${paraId} not in the list of parachain urls.`)
        }
    }

    ethChain(ethChainId: number): AbstractProvider {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            return this.#ethChains[ethChainKey]
        }

        const { ethChains } = this.config.ethereum
        if (ethChainKey in ethChains) {
            const url = ethChains[ethChainKey]
            let provider: AbstractProvider
            if (typeof url === "string") {
                if (url.startsWith("http")) {
                    provider = new JsonRpcProvider(url)
                } else {
                    provider = new WebSocketProvider(url)
                }
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
        return this.ethChain(this.config.ethereum.ethChainId)
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
        // clean up contract listeners
        if (this.#beefyClient) await this.beefyClient().removeAllListeners()
        if (this.#gateway) await this.gateway().removeAllListeners()

        // clean up etheruem
        for (const ethChainKey of Object.keys(this.config.ethereum.ethChains)) {
            if (
                typeof this.config.ethereum.ethChains[ethChainKey] === "string" &&
                this.#ethChains[ethChainKey]
            ) {
                this.#ethChains[ethChainKey].destroy()
            }
        }
        // clean up polkadot
        if (this.#relaychain) {
            await this.#relaychain.disconnect()
        }

        for (const paraId of Object.keys(this.#polkadotParachains)) {
            await this.#polkadotParachains[Number(paraId)].disconnect()
        }
        for (const paraId of Object.keys(this.#kusamaParachains)) {
            await this.#kusamaParachains[Number(paraId)].disconnect()
        }
    }
}

export * as toPolkadot from "./toPolkadot"
export * as toPolkadotV2 from "./toPolkadot_v2"
export * as toEthereum from "./toEthereum"
export * as toEthereumV2 from "./toEthereum_v2"
export * as utils from "./utils"
export * as status from "./status"
export * as assets from "./assets"
export * as assetsV2 from "./assets_v2"
export * as environment from "./environment"
export * as subscan from "./subscan"
export * as history from "./history"
export * as historyV2 from "./history_v2"
export * as subsquid from "./subsquid"
export * as forKusama from "./forKusama"
export * as toEthereumFromEVMV2 from "./toEthereumFromEVM_v2"
export * as parachains from "./parachains"
export * as toEthereumSnowbridgeV2 from "./toEthereumSnowbridgeV2"
