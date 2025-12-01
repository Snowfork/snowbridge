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
import { SNOWBRIDGE_ENV } from "./environment"

export * as toPolkadotV2 from "./toPolkadot_v2"
export * as toEthereumV2 from "./toEthereum_v2"
export * as utils from "./utils"
export * as status from "./status"
export * as assetsV2 from "./assets_v2"
export * as environment from "./environment"
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
    graphqlApiUrl: string
    monitorChains?: number[]
}

export class Context {
    config: Config

    // Ethereum
    #ethChains: EthereumChains
    #gateway?: IGatewayV1
    #gatewayV2?: IGatewayV2
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
            let options: any = {
                noInitWarn: true,
                provider: url.startsWith("http") ? new HttpProvider(url) : new WsProvider(url),
            }
            if (paraId === this.config.polkadot.bridgeHubParaId) {
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

    gateway(): IGatewayV1 {
        if (this.#gateway) {
            return this.#gateway
        }
        return IGatewayV1__factory.connect(this.config.appContracts.gateway, this.ethereum())
    }

    gatewayV2(): IGatewayV2 {
        if (this.#gatewayV2) {
            return this.#gatewayV2
        }
        return IGatewayV2__factory.connect(this.config.appContracts.gateway, this.ethereum())
    }

    beefyClient(): BeefyClient {
        if (this.#beefyClient) {
            return this.#beefyClient
        }
        return BeefyClient__factory.connect(this.config.appContracts.beefy, this.ethereum())
    }

    graphqlApiUrl(): string {
        return this.config.graphqlApiUrl
    }

    monitorChains(): number[] | undefined {
        return this.config.monitorChains
    }

    async destroyContext(): Promise<void> {
        // clean up contract listeners
        if (this.#beefyClient) await this.beefyClient().removeAllListeners()
        if (this.#gateway) await this.gateway().removeAllListeners()
        if (this.#gatewayV2) await this.gatewayV2().removeAllListeners()

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

export function contextConfigFor(
    env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {}),
): Config {
    if (!(env in SNOWBRIDGE_ENV)) {
        throw Error(`Unknown environment '${env}'.`)
    }
    const {
        ethChainId,
        config: {
            ASSET_HUB_PARAID,
            BEACON_HTTP_API,
            BEEFY_CONTRACT,
            BRIDGE_HUB_PARAID,
            ETHEREUM_CHAINS,
            GATEWAY_CONTRACT,
            PARACHAINS,
            RELAY_CHAIN_URL,
            GRAPHQL_API_URL,
            TO_MONITOR_PARACHAINS,
        },
        kusamaConfig,
    } = SNOWBRIDGE_ENV[env]

    let kusama:
        | {
              assetHubParaId: number
              bridgeHubParaId: number
              parachains: { [paraId: string]: string }
          }
        | undefined = undefined

    if (kusamaConfig) {
        const kusamaParachains: { [paraId: string]: string } = {}
        kusamaParachains[kusamaConfig?.BRIDGE_HUB_PARAID.toString()] =
            kusamaConfig?.PARACHAINS[BRIDGE_HUB_PARAID.toString()]
        kusamaParachains[kusamaConfig?.ASSET_HUB_PARAID.toString()] =
            kusamaConfig?.PARACHAINS[ASSET_HUB_PARAID.toString()]

        kusama = {
            assetHubParaId: kusamaConfig.ASSET_HUB_PARAID,
            bridgeHubParaId: kusamaConfig.BRIDGE_HUB_PARAID,
            parachains: kusamaParachains,
        }
    }

    return {
        environment: env,
        ethereum: {
            ethChainId,
            ethChains: ETHEREUM_CHAINS,
            beacon_url: BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: ASSET_HUB_PARAID,
            bridgeHubParaId: BRIDGE_HUB_PARAID,
            parachains: PARACHAINS,
            relaychain: RELAY_CHAIN_URL,
        },
        kusama,
        appContracts: {
            gateway: GATEWAY_CONTRACT,
            beefy: BEEFY_CONTRACT,
        },
        graphqlApiUrl: GRAPHQL_API_URL,
        monitorChains: TO_MONITOR_PARACHAINS,
    }
}

export function contextConfigOverrides(input: Config): Config {
    let config = { ...input }
    let injectedEthChains: { [ethChainId: string]: string | AbstractProvider } = {}
    for (const ethChainIdKey of Object.keys(input.ethereum.ethChains)) {
        if (
            process.env[`ETHEREUM_RPC_URL_${ethChainIdKey}`] ||
            process.env[`NEXT_PUBLIC_ETHEREUM_RPC_URL_${ethChainIdKey}`]
        ) {
            injectedEthChains[ethChainIdKey] =
                process.env[`ETHEREUM_RPC_URL_${ethChainIdKey}`] ||
                (process.env[`NEXT_PUBLIC_ETHEREUM_RPC_URL_${ethChainIdKey}`] as string)
            continue
        }
        injectedEthChains[ethChainIdKey] = input.ethereum.ethChains[ethChainIdKey]
    }
    config.ethereum.ethChains = injectedEthChains
    config.ethereum.beacon_url =
        process.env["BEACON_RPC_URL"] ||
        process.env["NEXT_PUBLIC_BEACON_RPC_URL"] ||
        input.ethereum.beacon_url

    let injectedParachains: { [paraId: string]: string } = {}
    for (const paraIdKey of Object.keys(input.polkadot.parachains)) {
        if (
            process.env[`PARACHAIN_RPC_URL_${paraIdKey}`] ||
            process.env[`NEXT_PUBLIC_PARACHAIN_RPC_URL_${paraIdKey}`]
        ) {
            injectedParachains[paraIdKey] = (process.env[`PARACHAIN_RPC_URL_${paraIdKey}`] ||
                process.env[`NEXT_PUBLIC_PARACHAIN_RPC_URL_${paraIdKey}`]) as string
            continue
        }
        injectedParachains[paraIdKey] = input.polkadot.parachains[paraIdKey]
    }
    config.polkadot.parachains = injectedParachains
    config.polkadot.relaychain =
        process.env["RELAY_CHAIN_RPC_URL"] ||
        process.env["NEXT_PUBLIC_RELAY_CHAIN_RPC_URL"] ||
        input.polkadot.relaychain

    return config
}
