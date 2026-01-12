// import '@polkadot/api-augment/polkadot'
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { AbstractProvider, JsonRpcProvider, WebSocketProvider } from "ethers"
import {
    BeefyClient,
    BeefyClient__factory,
    IGatewayV1,
    IGatewayV1__factory,
    IGatewayV2,
    IGatewayV2__factory,
} from "@snowbridge/contract-types"
import { Environment } from "@snowbridge/base-types"

export * as toPolkadotV2 from "./toPolkadot_v2"
export * as toEthereumV2 from "./toEthereum_v2"
export * as utils from "./utils"
export * as status from "./status"
export * as assetsV2 from "./assets_v2"
export * as history from "./history"
export * as subsquid from "./subsquid"
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

interface Parachains {
    [paraId: string]: ApiPromise
}
interface EthereumChains {
    [ethChainId: string]: AbstractProvider
}

export class Context {
    environment: Environment

    // Ethereum
    #ethChains: EthereumChains
    #gateway?: IGatewayV1
    #gatewayV2?: IGatewayV2
    #beefyClient?: BeefyClient

    // Substrate
    #polkadotParachains: Parachains
    #kusamaParachains: Parachains
    #relaychain?: ApiPromise

    constructor(environment: Environment) {
        this.environment = environment
        this.#polkadotParachains = {}
        this.#kusamaParachains = {}
        this.#ethChains = {}
    }

    async relaychain(): Promise<ApiPromise> {
        if (this.#relaychain) {
            return this.#relaychain
        }
        const url = this.environment.relaychainUrl
        console.log("Connecting to the relaychain.")
        this.#relaychain = await ApiPromise.create({
            noInitWarn: true,
            provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
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
            return this.#polkadotParachains[paraIdKey]
        }
        const { parachains } = this.environment
        if (paraIdKey in parachains) {
            const url = parachains[paraIdKey]
            console.log("Connecting to parachain ", paraIdKey, url)
            let options: any = {
                noInitWarn: true,
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
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
            const api = await ApiPromise.create(options)
            const onChainParaId = (
                await api.query.parachainInfo.parachainId()
            ).toPrimitive() as number
            if (onChainParaId !== paraId) {
                console.warn(
                    `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}, url=${url}`,
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
        if (!this.environment.kusama) {
            throw Error(`Kusama config is not set.`)
        }
        const { parachains } = this.environment.kusama
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
                    `Parachain id configured does not match onchain value. Configured = ${paraId}, OnChain=${onChainParaId}, url=${url}`,
                )
            }
            this.#kusamaParachains[onChainParaId] = api
            console.log("Connected to Kusama parachain ", paraIdKey)
            return this.#kusamaParachains[onChainParaId]
        } else {
            throw Error(`Parachain id ${paraId} not in the list of parachain urls.`)
        }
    }

    setEthProvider(ethChainId: number, provider: AbstractProvider) {
        const ethChainKey = ethChainId.toString()
        if (ethChainKey in this.#ethChains) {
            this.#ethChains[ethChainKey].destroy()
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
        return this.ethChain(this.environment.ethChainId)
    }

    gateway(): IGatewayV1 {
        if (this.#gateway) {
            return this.#gateway
        }
        return IGatewayV1__factory.connect(this.environment.gatewayContract, this.ethereum())
    }

    gatewayV2(): IGatewayV2 {
        if (this.#gatewayV2) {
            return this.#gatewayV2
        }
        return IGatewayV2__factory.connect(this.environment.gatewayContract, this.ethereum())
    }

    beefyClient(): BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        return BeefyClient__factory.connect(this.environment.beefyContract, this.ethereum())
    }

    graphqlApiUrl(): string {
        return this.environment.indexerGraphQlUrl
    }

    async destroyContext(): Promise<void> {
        // clean up contract listeners
        if (this.#beefyClient) await this.beefyClient().removeAllListeners()
        if (this.#gateway) await this.gateway().removeAllListeners()
        if (this.#gatewayV2) await this.gatewayV2().removeAllListeners()

        // clean up etheruem
        for (const ethChainKey of Object.keys(this.environment.ethereumChains)) {
            if (
                typeof this.environment.ethereumChains[ethChainKey] === "string" &&
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
