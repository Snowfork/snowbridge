import { ApiPromise } from "@polkadot/api"
import { AbstractProvider } from "ethers"

export type ERC20Metadata = {
    token: string
    name: string
    symbol: string
    decimals: number
    foreignId?: string
}

export type EthereumChain = {
    chainId: number
    id: string
    evmParachainId?: number
    assets: ERC20MetadataMap
    precompile?: `0x${string}`
    xcDOT?: string
    xcTokenMap?: XC20TokenMap
}

export type AccountType = "AccountId20" | "AccountId32"

export type SubstrateAccount = {
    nonce: bigint
    consumers: bigint
    providers: bigint
    sufficients: bigint
    data: {
        free: bigint
        reserved: bigint
        frozen: bigint
    }
}

export type ChainProperties = {
    tokenSymbols: string
    tokenDecimals: number
    ss58Format: number
    isEthereum: boolean
    accountType: AccountType
    evmChainId?: number
    name: string
    specName: string
    specVersion: number
}

export type Parachain = {
    parachainId: number
    info: ChainProperties
    features: {
        hasPalletXcm: boolean
        hasDryRunApi: boolean
        hasTxPaymentApi: boolean
        hasDryRunRpc: boolean
        hasDotBalance: boolean
    }
    assets: AssetMap
    estimatedExecutionFeeDOT: bigint
    estimatedDeliveryFeeDOT: bigint
    xcDOT?: string
}

export type Asset = {
    token: string
    name: string
    minimumBalance: bigint
    symbol: string
    decimals: number
    isSufficient: boolean
    xc20?: string
    // Location on source Parachain
    location?: any
    // Location reanchored on AH
    locationOnAH?: any
    // Location reanchored on Ethereum
    locationOnEthereum?: any
    // For chains that use `Assets` pallet to manage local assets
    // the asset_id is normally represented as u32, but on Moonbeam,
    // it is u128, so use string here to avoid overflow
    assetId?: string
    // Identifier of the PNA
    foreignId?: string
}

export type RegistryOptions = {
    environment: string
    gatewayAddress: string
    ethChainId: number
    assetHubParaId: number
    bridgeHubParaId: number
    parachains: (string | ApiPromise)[]
    ethchains: (string | AbstractProvider)[]
    relaychain: string | ApiPromise
    bridgeHub: string | ApiPromise
    kusama?: KusamaOptions
    precompiles?: PrecompileMap
    assetOverrides?: AssetOverrideMap
}

export type KusamaOptions = {
    assetHubParaId: number
    bridgeHubParaId: number
    assetHub: string | ApiPromise
}

export type AssetRegistry = {
    environment: string
    gatewayAddress: string
    ethChainId: number
    assetHubParaId: number
    bridgeHubParaId: number
    relaychain: ChainProperties
    bridgeHub: ChainProperties
    ethereumChains: {
        [chainId: string]: EthereumChain
    }
    parachains: ParachainMap
    kusama: KusamaConfig | undefined
}

export type KusamaConfig = {
    assetHubParaId: number
    bridgeHubParaId: number
    parachains: ParachainMap
}

export interface PNAMap {
    [token: string]: {
        token: string
        foreignId: string
        ethereumlocation: any
    }
}

export interface AssetMap {
    [token: string]: Asset
}

export interface ParachainMap {
    [paraId: string]: Parachain
}

export interface PrecompileMap {
    [chainId: string]: `0x${string}`
}

export interface AssetOverrideMap {
    [paraId: string]: Asset[]
}

export interface XC20TokenMap {
    [xc20: string]: string
}

export interface ERC20MetadataMap {
    [token: string]: ERC20Metadata
}

export type SourceType = "substrate" | "ethereum"

export type Path = {
    type: SourceType
    id: string
    source: number
    destination: number
    asset: string
}

export type Source = {
    type: SourceType
    id: string
    key: string
    destinations: { [destination: string]: string[] }
}

export type TransferLocation = {
    id: string
    name: string
    key: string
    type: SourceType
    parachain?: Parachain
    ethChain?: EthereumChain
}

export const ETHER_TOKEN_ADDRESS = "0x0000000000000000000000000000000000000000"
