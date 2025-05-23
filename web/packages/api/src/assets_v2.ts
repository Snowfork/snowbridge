import { AbstractProvider, Contract, ethers } from "ethers"
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { isFunction } from "@polkadot/util"
import { SnowbridgeEnvironment } from "./environment"
import { Context } from "./index"
import {
    buildParachainERC20ReceivedXcmOnDestination,
    convertToXcmV3X1,
    DOT_LOCATION,
    dotLocationOnKusamaAssetHub,
    erc20Location,
    getTokenFromLocation,
} from "./xcmBuilder"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { getMoonbeamEvmAssetMetadata, getMoonbeamLocationBalance, toMoonbeamXC20 } from "./parachains/moonbeam"
import { MUSE_TOKEN_ID, MYTHOS_TOKEN_ID, getMythosLocationBalance } from "./parachains/mythos"

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
        hasXcmPaymentApi: boolean
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
    destinationFeeOverrides?: FeeOverrideMap
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

type KusamaConfig = {
    assetHubParaId: number
    bridgeHubParaId: number
    parachains: ParachainMap
}

export interface AssetMap {
    [token: string]: Asset
}

interface ParachainMap {
    [paraId: string]: Parachain
}

interface PrecompileMap {
    [chainId: string]: `0x${string}`
}

interface AssetOverrideMap {
    [paraId: string]: Asset[]
}

interface FeeOverrideMap {
    [paraId: string]: bigint
}

interface XC20TokenMap {
    [xc20: string]: string
}

interface ERC20MetadataMap {
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

export async function buildRegistry(options: RegistryOptions): Promise<AssetRegistry> {
    const {
        environment,
        parachains,
        ethchains,
        ethChainId,
        gatewayAddress,
        assetHubParaId,
        bridgeHubParaId,
        relaychain,
        bridgeHub,
        kusama,
        precompiles,
        destinationFeeOverrides,
    } = options

    let relayInfo: ChainProperties
    {
        let provider: ApiPromise
        if (typeof relaychain === "string") {
            provider = await ApiPromise.create({
                noInitWarn: true,
                provider: relaychain.startsWith("http")
                    ? new HttpProvider(relaychain)
                    : new WsProvider(relaychain),
            })
        } else {
            provider = relaychain
        }

        relayInfo = await chainProperties(provider)

        if (typeof relaychain === "string") {
            await provider.disconnect()
        }
    }

    let bridgeHubInfo: ChainProperties
    {
        let provider: ApiPromise
        if (typeof bridgeHub === "string") {
            provider = await ApiPromise.create({
                noInitWarn: true,
                provider: bridgeHub.startsWith("http")
                    ? new HttpProvider(bridgeHub)
                    : new WsProvider(bridgeHub),
            })
        } else {
            provider = bridgeHub
        }
        bridgeHubInfo = await chainProperties(provider)

        if (typeof bridgeHub === "string") {
            await provider.disconnect()
        }
    }

    // Connect to all substrate parachains.
    const providers: {
        [paraIdKey: string]: { parachainId: number; provider: ApiPromise; managed: boolean }
    } = {}
    {
        for (const { parachainId, provider, managed } of await Promise.all(
            parachains.map(async (parachain) => {
                let provider: ApiPromise
                let managed = false
                if (typeof parachain === "string") {
                    provider = await ApiPromise.create({
                        noInitWarn: true,
                        provider: parachain.startsWith("http")
                            ? new HttpProvider(parachain)
                            : new WsProvider(parachain),
                    })
                    managed = true
                } else {
                    provider = parachain
                }
                const parachainId = await getParachainId(provider)
                return { parachainId, provider, managed }
            })
        )) {
            providers[parachainId.toString()] = { parachainId, provider, managed }
        }
        if (!(assetHubParaId.toString() in providers)) {
            throw Error(
                `Could not resolve asset hub para id ${assetHubParaId} in the list of parachains provided.`
            )
        }
    }

    // Connect to all eth connections
    const ethProviders: {
        [chainId: string]: {
            chainId: number
            provider: AbstractProvider
            managed: boolean
            name: string
        }
    } = {}
    {
        for (const result of await Promise.all(
            ethchains.map(async (ethChain) => {
                let provider: AbstractProvider
                let managed = false
                if (typeof ethChain === "string") {
                    provider = ethers.getDefaultProvider(ethChain)
                    managed = true
                } else {
                    provider = ethChain
                }
                const network = await provider.getNetwork()
                return { chainId: Number(network.chainId), provider, managed, name: network.name }
            })
        )) {
            ethProviders[result.chainId.toString()] = result
        }
        if (!(ethChainId.toString() in ethProviders)) {
            throw Error(`Cannot find ethereum chain ${ethChainId} in the list of ethereum chains.`)
        }
    }

    const pnaOverrides = await indexPNAs(
        environment,
        bridgeHub as ApiPromise,
        providers[assetHubParaId].provider,
        ethProviders[ethChainId].provider,
        gatewayAddress,
        assetHubParaId
    )
    const assetOverrides = { ...options.assetOverrides, ...pnaOverrides }

    // Index parachains
    const paras: ParachainMap = {}
    for (const { parachainId, para } of await Promise.all(
        Object.keys(providers).map(async (parachainIdKey) => {
            const { parachainId, provider } = providers[parachainIdKey]
            const para = await indexParachain(
                provider,
                providers[assetHubParaId.toString()].provider,
                ethChainId,
                parachainId,
                assetHubParaId,
                assetOverrides ?? {},
                destinationFeeOverrides ?? {}
            )
            return { parachainId, para }
        })
    )) {
        paras[parachainId.toString()] = para
    }

    // Index Ethereum chain
    const ethChains: { [chainId: string]: EthereumChain } = {}
    for (const ethChainInfo of await Promise.all(
        Object.keys(ethProviders).map(async (ethChainKey) => {
            return indexEthChain(
                ethProviders[ethChainKey].provider,
                ethProviders[ethChainKey].chainId,
                ethProviders[ethChainKey].name,
                ethChainId,
                gatewayAddress,
                assetHubParaId,
                paras,
                precompiles ?? {}
            )
        })
    )) {
        ethChains[ethChainInfo.chainId.toString()] = ethChainInfo
    }

    // Dispose of all substrate connections
    await Promise.all(
        Object.keys(providers)
            .filter((parachainKey) => providers[parachainKey].managed)
            .map(async (parachainKey) => await providers[parachainKey].provider.disconnect())
    )

    // Dispose all eth connections
    Object.keys(ethProviders)
        .filter((parachainKey) => ethProviders[parachainKey].managed)
        .forEach((parachainKey) => ethProviders[parachainKey].provider.destroy())

    let kusamaConfig: KusamaConfig | undefined
    if (kusama) {
        let kusamaAssetHub = kusama.assetHub
        let provider: ApiPromise
        if (typeof kusamaAssetHub === "string") {
            provider = await ApiPromise.create({
                noInitWarn: true,
                provider: kusamaAssetHub.startsWith("http")
                    ? new HttpProvider(kusamaAssetHub)
                    : new WsProvider(kusamaAssetHub),
            })
        } else {
            provider = kusamaAssetHub
        }

        const kusamaPnaOverrides = await indexKusamaPNAs(
            bridgeHub as ApiPromise,
            providers[assetHubParaId].provider,
            provider,
            ethProviders[ethChainId].provider,
            gatewayAddress,
            assetHubParaId
        )
        let assetOverrides = { ...options.assetOverrides, ...kusamaPnaOverrides }

        const para = await indexParachain(
            provider,
            provider,
            ethChainId,
            kusama.assetHubParaId,
            kusama.assetHubParaId,
            assetOverrides ?? {},
            destinationFeeOverrides ?? {}
        )

        const kusamaParas: ParachainMap = {}
        kusamaParas[kusama.assetHubParaId] = para

        kusamaConfig = {
            parachains: kusamaParas,
            assetHubParaId: kusama.assetHubParaId,
            bridgeHubParaId: kusama.bridgeHubParaId,
        }
    }

    return {
        environment,
        ethChainId,
        gatewayAddress,
        assetHubParaId,
        bridgeHubParaId,
        relaychain: relayInfo,
        bridgeHub: bridgeHubInfo,
        ethereumChains: ethChains,
        parachains: paras,
        kusama: kusamaConfig,
    }
}

export function getEthereumTransferLocation(
    registry: AssetRegistry,
    ethChain: EthereumChain
): TransferLocation {
    if (!ethChain.evmParachainId) {
        return {
            id: "ethereum",
            name: "Ethereum",
            type: "ethereum",
            key: ethChain.chainId.toString(),
            ethChain,
        }
    } else {
        const evmChain = registry.parachains[ethChain.evmParachainId]
        return {
            id: ethChain.id,
            name: `${evmChain.info.name} (EVM)`,
            key: ethChain.chainId.toString(),
            type: "ethereum",
            ethChain,
            parachain: evmChain,
        }
    }
}

export function getSubstrateTransferLocation(parachain: Parachain): TransferLocation {
    return {
        id: parachain.info.specName,
        name: parachain.info.name,
        key: parachain.parachainId.toString(),
        type: "substrate",
        parachain,
    }
}

export function getTransferLocation(
    registry: AssetRegistry,
    sourceType: string,
    sourceKey: string
): TransferLocation {
    if (sourceType === "ethereum") {
        return getEthereumTransferLocation(registry, registry.ethereumChains[sourceKey])
    } else {
        return getSubstrateTransferLocation(registry.parachains[sourceKey])
    }
}

export function getTransferLocations(
    registry: AssetRegistry,
    filter?: (path: Path) => boolean
): Source[] {
    const ethChain = registry.ethereumChains[registry.ethChainId]
    const parachains = Object.keys(registry.parachains)
        .filter((p) => p !== registry.bridgeHubParaId.toString())
        .map((p) => registry.parachains[p])

    const pathFilter = filter ?? defaultPathFilter(registry.environment)

    const locations: Path[] = []
    for (const parachain of parachains) {
        const sourceAssets = Object.keys(ethChain.assets)
        const destinationAssets = Object.keys(parachain.assets)
        const commonAssets = new Set(
            sourceAssets.filter((sa) => destinationAssets.find((da) => da === sa))
        )
        for (const asset of commonAssets) {
            const p1: Path = {
                type: "ethereum",
                id: "ethereum",
                source: ethChain.chainId,
                destination: parachain.parachainId,
                asset,
            }
            if (pathFilter(p1)) {
                locations.push(p1)
            }
            const p2: Path = {
                type: "substrate",
                id: parachain.info.specName,
                source: parachain.parachainId,
                destination: ethChain.chainId,
                asset,
            }
            if (pathFilter(p2)) {
                locations.push(p2)
            }
            if (parachain.info.evmChainId && registry.ethereumChains[parachain.info.evmChainId]) {
                const p3: Path = {
                    type: "ethereum",
                    id: `${parachain.info.specName}_evm`,
                    source: parachain.info.evmChainId,
                    destination: ethChain.chainId,
                    asset,
                }
                if (pathFilter(p3)) {
                    locations.push(p3)
                }
            }
        }
    }
    const results: Source[] = []
    for (const location of locations) {
        let source = results.find(
            (s) =>
                s.type === location.type &&
                s.id === location.id &&
                s.key === location.source.toString()
        )
        if (!source) {
            source = {
                type: location.type,
                id: location.id,
                key: location.source.toString(),
                destinations: {},
            }
            results.push(source)
        }
        let destination: string[] = source.destinations[location.destination]
        if (!destination) {
            destination = []
            source.destinations[location.destination] = destination
        }
        destination.push(location.asset)
    }
    return results
}

export function fromEnvironment(
    { name, config, kusamaConfig, ethChainId }: SnowbridgeEnvironment,
    ethereumApiKey?: string
): RegistryOptions {
    let result: RegistryOptions = {
        environment: name,
        assetHubParaId: config.ASSET_HUB_PARAID,
        bridgeHubParaId: config.BRIDGE_HUB_PARAID,
        bridgeHub: config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()],
        relaychain: config.RELAY_CHAIN_URL,
        ethChainId,
        gatewayAddress: config.GATEWAY_CONTRACT,
        ethchains: Object.values(config.ETHEREUM_CHAINS).map((x) => x(ethereumApiKey ?? "")),
        parachains: Object.keys(config.PARACHAINS)
            .filter((paraId) => paraId !== config.BRIDGE_HUB_PARAID.toString())
            .map((paraId) => config.PARACHAINS[paraId]),
    }
    if (kusamaConfig) {
        result.kusama = {
            assetHubParaId: kusamaConfig.ASSET_HUB_PARAID,
            bridgeHubParaId: kusamaConfig.BRIDGE_HUB_PARAID,
            assetHub: kusamaConfig.PARACHAINS[config.ASSET_HUB_PARAID.toString()],
        }
    }
    addOverrides(name, result)
    return result
}

export async function fromContext(context: Context): Promise<RegistryOptions> {
    const { assetHubParaId, bridgeHubParaId } = context.config.polkadot
    const [bridgeHub, relaychain, network, gatewayAddress, parachains] = await Promise.all([
        context.bridgeHub(),
        context.relaychain(),
        context.ethereum().getNetwork(),
        context.gateway().getAddress(),
        Promise.all(
            context
                .parachains()
                .filter((paraId) => paraId !== context.config.polkadot.bridgeHubParaId)
                .map((paraId) => context.parachain(paraId))
        ),
    ])

    let result: RegistryOptions = {
        environment: context.config.environment,
        assetHubParaId,
        bridgeHubParaId,
        bridgeHub,
        relaychain,
        ethChainId: Number(network.chainId),
        gatewayAddress,
        ethchains: context.ethChains().map((ethChainId) => context.ethChain(ethChainId)),
        parachains,
    }

    if (context.config.kusama) {
        const kusamaAssetHub = await context.kusamaAssetHub()

        if (kusamaAssetHub) {
            const { assetHubParaId, bridgeHubParaId } = context.config.kusama
            result.kusama = {
                assetHubParaId,
                bridgeHubParaId,
                assetHub: kusamaAssetHub,
            }
        }
    }

    addOverrides(context.config.environment, result)
    return result
}

export async function getNativeAccount(
    provider: ApiPromise,
    account: string
): Promise<SubstrateAccount> {
    const accountData = (await provider.query.system.account(account)).toPrimitive() as any
    return {
        nonce: BigInt(accountData.nonce),
        consumers: BigInt(accountData.consumers),
        providers: BigInt(accountData.providers),
        sufficients: BigInt(accountData.sufficients),
        data: {
            free: BigInt(accountData.data.free),
            reserved: BigInt(accountData.data.reserved),
            frozen: BigInt(accountData.data.frozen),
        },
    }
}

export async function getNativeBalance(provider: ApiPromise, account: string): Promise<bigint> {
    const accountData = await getNativeAccount(provider, account)
    return accountData.data.free
}

export async function getLocationBalance(
    provider: ApiPromise,
    specName: string,
    location: any,
    account: string,
    pnaAssetId?: any
): Promise<bigint> {
    switch (specName) {
        case "basilisk":
        case "hydradx": {
            const paraAssetId = (
                await provider.query.assetRegistry.locationAssets(convertToXcmV3X1(location))
            ).toPrimitive()
            if (!paraAssetId) {
                throw Error(`DOT not registered for spec ${specName}.`)
            }
            const accountData = (
                await provider.query.tokens.accounts(account, paraAssetId)
            ).toPrimitive() as any
            return BigInt(accountData?.free ?? 0n)
        }
        case "penpal-parachain":
        case "asset-hub-paseo":
        case "westmint":
        case "statemine":
        case "statemint": {
            let accountData: any
            if (pnaAssetId) {
                accountData = (
                    await provider.query.assets.account(pnaAssetId, account)
                ).toPrimitive() as any
            } else {
                accountData = (
                    await provider.query.foreignAssets.account(location, account)
                ).toPrimitive() as any
            }
            return BigInt(accountData?.balance ?? 0n)
        }
        case "bifrost":
        case "bifrost_paseo":
        case "bifrost_polkadot": {
            const paraAssetId = (
                await provider.query.assetRegistry.locationToCurrencyIds(location)
            ).toPrimitive()
            if (!paraAssetId) {
                throw Error(`DOT not registered for spec ${specName}.`)
            }
            const accountData = (
                await provider.query.tokens.accounts(account, paraAssetId)
            ).toPrimitive() as any
            return BigInt(accountData?.free ?? 0n)
        }
        case "moonriver":
        case "moonbeam": {
            return await getMoonbeamLocationBalance(
                pnaAssetId,
                location,
                provider,
                specName,
                account
            )
        }
        case "muse":
        case "mythos": {
            return await getMythosLocationBalance(location, provider, specName, account)
        }
        default:
            throw Error(
                `Cannot get balance for spec ${specName}. Location = ${JSON.stringify(location)}`
            )
    }
}

export function getDotBalance(
    provider: ApiPromise,
    specName: string,
    account: string
): Promise<bigint> {
    switch (specName) {
        case "asset-hub-paseo":
        case "westmint":
        case "statemint": {
            return getNativeBalance(provider, account)
        }
        case "statemine": {
            return getLocationBalance(provider, specName, dotLocationOnKusamaAssetHub, account)
        }
        default:
            return getLocationBalance(provider, specName, DOT_LOCATION, account)
    }
}

export function getTokenBalance(
    provider: ApiPromise,
    specName: string,
    account: string,
    ethChainId: number,
    tokenAddress: string,
    asset?: Asset
) {
    return getLocationBalance(
        provider,
        specName,
        asset?.location ?? erc20Location(ethChainId, tokenAddress),
        account,
        asset?.assetId
    )
}

export async function getParachainId(parachain: ApiPromise): Promise<number> {
    const sourceParachainEncoded = await parachain.query.parachainInfo.parachainId()
    return Number(sourceParachainEncoded.toPrimitive())
}

export async function calculateDestinationFee(provider: ApiPromise, destinationXcm: any) {
    const weight = (
        await provider.call.xcmPaymentApi.queryXcmWeight(destinationXcm)
    ).toPrimitive() as any
    if (!weight.ok) {
        throw Error(`Can not query XCM Weight.`)
    }

    let feeInDot: any
    feeInDot = (
        await provider.call.xcmPaymentApi.queryWeightToAssetFee(weight.ok, {
            v4: { parents: 1, interior: "Here" },
        })
    ).toPrimitive() as any
    // For compatibility with Westend, which has XCMV5 enabled.
    if (!feeInDot.ok) {
        feeInDot = (
            await provider.call.xcmPaymentApi.queryWeightToAssetFee(weight.ok, {
                v5: { parents: 1, interior: "Here" },
            })
        ).toPrimitive() as any
        if (!feeInDot.ok) throw Error(`Can not convert weight to fee in DOT.`)
    }
    const executionFee = BigInt(feeInDot.ok.toString())

    return executionFee
}

export async function calculateDeliveryFee(
    provider: ApiPromise,
    parachainId: number,
    destinationXcm: any
) {
    const result = (
        await provider.call.xcmPaymentApi.queryDeliveryFees(
            { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } },
            destinationXcm
        )
    ).toPrimitive() as any
    if (!result.ok) {
        throw Error(`Can not query XCM Weight.`)
    }
    let dotAsset = undefined
    for (const asset of result.ok.v4) {
        if (asset.id.parents === 1 && asset.id.interior.here === null) {
            dotAsset = asset
        }
    }
    if (!dotAsset) {
        console.info("Could not find DOT in result", result)
        throw Error(`Can not query XCM Weight.`)
    }

    const deliveryFee = BigInt(dotAsset.fun.fungible.toString())

    return deliveryFee
}

export function padFeeByPercentage(fee: bigint, padPercent: bigint) {
    if (padPercent < 0 || padPercent > 100) {
        throw Error(`padPercent ${padPercent} not in range of 0 to 100.`)
    }
    return fee * ((100n + padPercent) / 100n)
}

async function chainProperties(provider: ApiPromise): Promise<ChainProperties> {
    const [properties, name] = await Promise.all([
        provider.rpc.system.properties(),
        provider.rpc.system.chain(),
    ])
    const tokenSymbols = properties.tokenSymbol.unwrapOrDefault().at(0)?.toString()
    const tokenDecimals = properties.tokenDecimals.unwrapOrDefault().at(0)?.toNumber()
    const isEthereum = properties.isEthereum.toPrimitive()
    const ss58Format =
        (provider.consts.system.ss58Prefix.toPrimitive() as number) ??
        properties.ss58Format.unwrapOr(null)?.toNumber()
    const { specName, specVersion } = provider.consts.system.version.toJSON() as any
    const accountType = provider.registry.getDefinition("AccountId")

    let evmChainId: number | undefined
    if (provider.query.evmChainId) {
        evmChainId = Number((await provider.query.evmChainId.chainId()).toPrimitive())
    } else if (provider.query.ethereumChainId) {
        evmChainId = Number((await provider.query.ethereumChainId.chainId()).toPrimitive())
    } else {
        evmChainId = undefined
    }

    if (accountType !== "AccountId20" && accountType !== "AccountId32") {
        throw Error(`Unknown account type ${accountType} for runtime ${specName}.`)
    }
    return {
        tokenSymbols: String(tokenSymbols),
        tokenDecimals: Number(tokenDecimals),
        ss58Format,
        isEthereum,
        accountType,
        evmChainId,
        name: name.toPrimitive(),
        specName,
        specVersion,
    }
}

async function indexParachainAssets(provider: ApiPromise, ethChainId: number, specName: string) {
    const assets: AssetMap = {}
    let xcDOT: string | undefined
    switch (specName) {
        case "basilisk":
        case "hydradx": {
            const entries = await provider.query.assetRegistry.assetLocations.entries()
            for (const [id, value] of entries) {
                const location: any = value.toJSON()
                const token = getTokenFromLocation(location, ethChainId)
                if (!token) {
                    continue
                }

                const assetId = Number(id.args.at(0)?.toString())
                const asset: any = (
                    await provider.query.assetRegistry.assets(assetId)
                ).toPrimitive()

                assets[token] = {
                    token,
                    name: String(asset.name ?? ""),
                    minimumBalance: BigInt(asset.existentialDeposit),
                    symbol: String(asset.symbol ?? ""),
                    decimals: Number(asset.decimals),
                    isSufficient: Boolean(asset.isSufficient),
                }
            }
            break
        }
        case "asset-hub-paseo":
        case "westmint":
        case "penpal-parachain":
        case "statemine":
        case "statemint": {
            const entries = await provider.query.foreignAssets.asset.entries()
            for (const [key, value] of entries) {
                const location: any = key.args.at(0)?.toJSON()
                if (!location) {
                    console.warn(`Could not convert ${key.toHuman()} to location for ${specName}.`)
                    continue
                }
                const token = getTokenFromLocation(location, ethChainId)
                if (!token) {
                    continue
                }

                const asset: any = value.toJSON()
                const assetMetadata: any = (
                    await provider.query.foreignAssets.metadata(location)
                ).toPrimitive()

                assets[token] = {
                    token,
                    name: String(assetMetadata.name),
                    minimumBalance: BigInt(asset.minBalance),
                    symbol: String(assetMetadata.symbol),
                    decimals: Number(assetMetadata.decimals),
                    isSufficient: Boolean(asset.isSufficient),
                }
            }
            break
        }
        case "bifrost":
        case "bifrost_paseo":
        case "bifrost_polkadot": {
            const entries = await provider.query.assetRegistry.currencyIdToLocations.entries()
            for (const [key, value] of entries) {
                const location: any = value.toJSON()
                const token = getTokenFromLocation(location, ethChainId)
                if (!token) {
                    continue
                }

                const assetId: any = key.args.at(0)
                const asset: any = (
                    await provider.query.assetRegistry.currencyMetadatas(assetId)
                ).toPrimitive()

                assets[token] = {
                    token,
                    name: String(asset.name),
                    minimumBalance: BigInt(asset.minimalBalance),
                    symbol: String(asset.symbol),
                    decimals: Number(asset.decimals),
                    isSufficient: false,
                }
            }
            break
        }
        case "moonriver":
        case "moonbeam": {
            const foreignEntries = await provider.query.evmForeignAssets.assetsById.entries()
            for (const [key, value] of foreignEntries) {
                const location = value.toJSON() as any

                const assetId = BigInt(key.args.at(0)?.toPrimitive() as any)
                const xc20 = toMoonbeamXC20(assetId)

                if (location.parents === 1 && location.interior.here !== undefined) {
                    xcDOT = xc20
                }
                const token = getTokenFromLocation(location, ethChainId)
                if (!token) {
                    continue
                }
                // we found the asset in pallet-assets so we can skip evmForeignAssets.
                if (assets[token]) {
                    continue
                }

                const symbol = await getMoonbeamEvmAssetMetadata(provider, "symbol", xc20)
                const name = await getMoonbeamEvmAssetMetadata(provider, "name", xc20)
                const decimals = await getMoonbeamEvmAssetMetadata(provider, "decimals", xc20)

                assets[token] = {
                    token,
                    name: String(name),
                    minimumBalance: 1n,
                    symbol: String(symbol),
                    decimals: Number(decimals),
                    isSufficient: true,
                    xc20,
                }
            }
            break
        }
    }
    return {
        assets,
        xcDOT,
    }
}

async function indexParachain(
    provider: ApiPromise,
    assetHub: ApiPromise,
    ethChainId: number,
    parachainId: number,
    assetHubParaId: number,
    assetOverrides: AssetOverrideMap,
    destinationFeeOverrides: FeeOverrideMap
): Promise<Parachain> {
    const info = await chainProperties(provider)

    const { assets, xcDOT } = await indexParachainAssets(provider, ethChainId, info.specName)
    const parachainIdKey = parachainId.toString()
    if (parachainIdKey in assetOverrides) {
        for (const asset of assetOverrides[parachainIdKey]) {
            assets[asset.token.toLowerCase()] = asset
        }
    }

    if (Object.keys(assets).length === 0) {
        console.warn(
            `Cannot discover assets for ${info.specName} (parachain ${parachainId}). Please add a handler for that runtime or add overrides.`
        )
    }

    const hasPalletXcm = isFunction(provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen)
    const hasDryRunRpc = isFunction(provider.rpc.system?.dryRun)
    const hasDryRunApi =
        isFunction(provider.call.dryRunApi?.dryRunCall) &&
        isFunction(provider.call.dryRunApi?.dryRunXcm)
    const hasTxPaymentApi = isFunction(provider.call.transactionPaymentApi?.queryInfo)
    const hasXcmPaymentApi =
        isFunction(provider.call.xcmPaymentApi?.queryXcmWeight) &&
        isFunction(provider.call.xcmPaymentApi?.queryDeliveryFees) &&
        isFunction(provider.call.xcmPaymentApi?.queryWeightToAssetFee)

    // test getting balances
    let hasDotBalance = true
    try {
        await getDotBalance(
            provider,
            info.specName,
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000"
        )
    } catch (err) {
        console.warn(`Spec ${info.specName} does not support dot ${err}`)
        hasDotBalance = false
    }

    await getNativeBalance(
        provider,
        info.accountType === "AccountId32"
            ? "0x0000000000000000000000000000000000000000000000000000000000000000"
            : "0x0000000000000000000000000000000000000000"
    )

    let estimatedExecutionFeeDOT = 0n
    let estimatedDeliveryFeeDOT = 0n
    if (parachainId !== assetHubParaId) {
        const destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
            provider.registry,
            ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        estimatedDeliveryFeeDOT = await calculateDeliveryFee(assetHub, parachainId, destinationXcm)
        if (hasXcmPaymentApi) {
            estimatedExecutionFeeDOT = await calculateDestinationFee(provider, destinationXcm)
        } else {
            if (!(parachainIdKey in destinationFeeOverrides)) {
                throw Error(
                    `Parachain ${parachainId} cannot fetch the destination fee and needs a fee override.`
                )
            }
            estimatedExecutionFeeDOT = destinationFeeOverrides[parachainIdKey]
        }
    }
    return {
        parachainId,
        features: {
            hasPalletXcm,
            hasDryRunApi,
            hasTxPaymentApi,
            hasDryRunRpc,
            hasXcmPaymentApi,
            hasDotBalance,
        },
        info,
        xcDOT,
        assets,
        estimatedExecutionFeeDOT,
        estimatedDeliveryFeeDOT,
    }
}

async function indexEthChain(
    provider: AbstractProvider,
    networkChainId: number,
    networkName: string,
    ethChainId: number,
    gatewayAddress: string,
    assetHubParaId: number,
    parachains: ParachainMap,
    precompiles: PrecompileMap
): Promise<EthereumChain> {
    const id = networkName !== "unknown" ? networkName : undefined
    if (networkChainId == ethChainId) {
        // Asset Hub and get meta data
        const assetHub = parachains[assetHubParaId.toString()]
        const gateway = IGateway__factory.connect(gatewayAddress, provider)

        const assets: ERC20MetadataMap = {}
        for (const token in assetHub.assets) {
            if (!(await gateway.isTokenRegistered(token))) {
                console.warn(`Token ${token} is not registered with the gateway.`)
            }
            if (token === ETHER_TOKEN_ADDRESS) {
                assets[token] = {
                    token: assetHub.assets[token].token,
                    name: assetHub.assets[token].name,
                    symbol: assetHub.assets[token].symbol,
                    decimals: assetHub.assets[token].decimals,
                }
            } else {
                assets[token] = await assetErc20Metadata(provider, token, gatewayAddress)
            }
        }
        if ((await provider.getCode(gatewayAddress)) === undefined) {
            throw Error(
                `Could not fetch code for gatway address ${gatewayAddress} on ethereum chain ${networkChainId}.`
            )
        }
        return {
            chainId: networkChainId,
            assets,
            id: id ?? `chain_${networkChainId}`,
        }
    } else {
        let evmParachainChain: Parachain | undefined
        for (const paraId in parachains) {
            const parachain = parachains[paraId]
            if (parachain.info.evmChainId === networkChainId) {
                evmParachainChain = parachain
                break
            }
        }
        if (!evmParachainChain) {
            throw Error(`Could not find evm chain ${networkChainId} in the list of parachains.`)
        }
        const xcTokenMap: XC20TokenMap = {}
        const assets: ERC20MetadataMap = {}
        for (const token in evmParachainChain.assets) {
            const xc20 = evmParachainChain.assets[token].xc20
            if (!xc20) {
                continue
            }
            const asset = await assetErc20Metadata(provider, xc20.toLowerCase())
            xcTokenMap[token] = xc20
            assets[xc20] = asset
        }
        const paraId = evmParachainChain.parachainId.toString()
        if (!(paraId in precompiles)) {
            throw Error(
                `No precompile configured for parachain ${paraId} (ethereum chain ${networkChainId}).`
            )
        }
        const precompile = precompiles[paraId]
        if ((await provider.getCode(precompile)) === undefined) {
            throw Error(
                `Could not fetch code for ${precompile} on parachain ${paraId} (ethereum chain ${networkChainId}).`
            )
        }
        if (!evmParachainChain.xcDOT) {
            throw Error(`Could not find DOT XC20 address for evm chain ${networkChainId}.`)
        }
        const xc20DOTAsset: ERC20Metadata = await assetErc20Metadata(
            provider,
            evmParachainChain.xcDOT
        )
        assets[evmParachainChain.xcDOT] = xc20DOTAsset

        return {
            chainId: networkChainId,
            evmParachainId: evmParachainChain.parachainId,
            assets,
            precompile,
            xcDOT: evmParachainChain.xcDOT,
            xcTokenMap,
            id: id ?? `evm_${evmParachainChain.info.specName}`,
        }
    }
}

const ERC20_METADATA_ABI = [
    {
        type: "function",
        name: "decimals",
        inputs: [],
        outputs: [
            {
                name: "",
                type: "uint8",
                internalType: "uint8",
            },
        ],
        stateMutability: "view",
    },
    {
        type: "function",
        name: "name",
        inputs: [],
        outputs: [
            {
                name: "",
                type: "string",
                internalType: "string",
            },
        ],
        stateMutability: "view",
    },
    {
        type: "function",
        name: "symbol",
        inputs: [],
        outputs: [
            {
                name: "",
                type: "string",
                internalType: "string",
            },
        ],
        stateMutability: "view",
    },
]

async function assetErc20Metadata(
    provider: AbstractProvider,
    token: string,
    gateway?: string
): Promise<ERC20Metadata> {
    const erc20Metadata = new Contract(token, ERC20_METADATA_ABI, provider)
    const [name, symbol, decimals] = await Promise.all([
        erc20Metadata.name(),
        erc20Metadata.symbol(),
        erc20Metadata.decimals(),
    ])
    let metadata: any = {
        token,
        name: String(name),
        symbol: String(symbol),
        decimals: Number(decimals),
    }
    if (gateway) {
        let gatewayCon = IGateway__factory.connect(gateway, provider)
        let tokenId = await gatewayCon.queryForeignTokenID(token)
        if (tokenId != "0x0000000000000000000000000000000000000000000000000000000000000000") {
            metadata.foreignId = tokenId
        }
    }
    return metadata
}

function addOverrides(envName: string, result: RegistryOptions) {
    switch (envName) {
        case "paseo_sepolia": {
            // Add override for mythos token and add precompile for moonbeam
            result.destinationFeeOverrides = {
                "3369": 200_000_000_000n,
            }
            result.assetOverrides = {
                "3369": [
                    {
                        token: MUSE_TOKEN_ID.toLowerCase(),
                        name: "Muse",
                        minimumBalance: 10_000_000_000_000_000n,
                        symbol: "MUSE",
                        decimals: 18,
                        isSufficient: true,
                    },
                ],
            }
            break
        }
        case "polkadot_mainnet": {
            // Add override for mythos token and add precompile for moonbeam
            result.precompiles = { "2004": "0x000000000000000000000000000000000000081a" }
            result.destinationFeeOverrides = {
                "3369": 100_000_000n,
            }
            result.assetOverrides = {
                "3369": [
                    {
                        token: MYTHOS_TOKEN_ID.toLowerCase(),
                        name: "Mythos",
                        minimumBalance: 10_000_000_000_000_000n,
                        symbol: "MYTH",
                        decimals: 18,
                        isSufficient: true,
                    },
                ],
            }
            break
        }
        case "westend_sepolia": {
            result.assetOverrides = {}
            break
        }
        case "local_e2e": {
            result.assetOverrides = {
                "1000": [
                    {
                        token: "0xDe45448Ca2d57797c0BEC0ee15A1E42334744219".toLowerCase(),
                        name: "wnd",
                        minimumBalance: 1n,
                        symbol: "wnd",
                        decimals: 18,
                        isSufficient: true,
                        location: DOT_LOCATION,
                    },
                    {
                        token: "0xD8597EB7eF761E3315623EdFEe9DEfcBACd72e8b".toLowerCase(),
                        name: "pal-2",
                        minimumBalance: 1n,
                        symbol: "pal-2",
                        decimals: 18,
                        isSufficient: true,
                        location: {
                            parents: 1,
                            interior: {
                                x3: [
                                    { parachain: 2000 },
                                    { palletInstance: 50 },
                                    { generalIndex: 2 },
                                ],
                            },
                        },
                    },
                ],
                "2000": [
                    {
                        token: "0xD8597EB7eF761E3315623EdFEe9DEfcBACd72e8b".toLowerCase(),
                        name: "pal-2",
                        minimumBalance: 1n,
                        symbol: "pal-2",
                        decimals: 18,
                        isSufficient: true,
                        assetId: "2",
                        location: {
                            parents: 0,
                            interior: { x2: [{ palletInstance: 50 }, { generalIndex: 2 }] },
                        },
                        locationOnAH: {
                            parents: 1,
                            interior: {
                                x3: [
                                    { parachain: 2000 },
                                    { palletInstance: 50 },
                                    { generalIndex: 2 },
                                ],
                            },
                        },
                        locationOnEthereum: {
                            parents: 1,
                            interior: {
                                x4: [
                                    {
                                        globalConsensus: {
                                            byGenesis:
                                                "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e",
                                        },
                                    },
                                    { parachain: 2000 },
                                    { palletInstance: 50 },
                                    { generalIndex: 2 },
                                ],
                            },
                        },
                    },
                ],
            }
            break
        }
    }
}

function defaultPathFilter(envName: string): (_: Path) => boolean {
    switch (envName) {
        case "westend_sepolia": {
            return (path: Path) => {
                // Frequency
                if (path.asset === "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620") {
                    return false
                }
                return true
            }
        }
        case "paseo_sepolia":
            return (path: Path) => {
                // Disallow MUSE to any location but 3369
                if (
                    path.asset === MUSE_TOKEN_ID &&
                    ((path.destination !== 3369 && path.type === "ethereum") ||
                        (path.source !== 3369 && path.type === "substrate"))
                ) {
                    return false
                }
                return true
            }
        case "polkadot_mainnet":
            return (path: Path) => {
                // Disallow MYTH to any location but 3369
                if (
                    path.asset === MYTHOS_TOKEN_ID &&
                    // TODO: Disable Mythos to Eth until mythos is ready to enable
                    path.destination !== 3369
                ) {
                    return false
                }

                // Disable stable coins in the UI from Ethereum to Polkadot
                if (
                    (path.asset === "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" || // USDC
                        path.asset === "0xdac17f958d2ee523a2206206994597c13d831ec7" || // USDT
                        path.asset === "0x9d39a5de30e57443bff2a8307a4256c8797a3497" || // Staked USDe
                        path.asset === "0xa3931d71877c0e7a3148cb7eb4463524fec27fbd" || // Savings USD
                        path.asset === "0x6b175474e89094c44da98b954eedeac495271d0f") && // DAI
                    path.destination === 2034 // Hydration
                ) {
                    return false
                }
                return true
            }

        default:
            return (_: Path) => true
    }
}

async function indexPNAs(
    environment: string,
    bridgehub: ApiPromise,
    assethub: ApiPromise,
    ethereum: AbstractProvider,
    gatewayAddress: string,
    assetHubParaId: number
): Promise<AssetOverrideMap> {
    let pnas: Asset[] = []
    let gateway = IGateway__factory.connect(gatewayAddress, ethereum)
    const entries = await bridgehub.query.ethereumSystem.nativeToForeignId.entries()
    for (const [key, value] of entries) {
        const location: any = key.args.at(0)?.toJSON()
        if (!location) {
            console.warn(`Could not convert ${key.toHuman()} to location`)
            continue
        }
        const locationOnAH: any = bridgeablePNAsOnPolkadotAH(environment, location, assetHubParaId)
        if (!locationOnAH) {
            console.warn(`Location ${JSON.stringify(location)} is not bridgeable on assethub`)
            continue
        }
        const tokenId = (value.toPrimitive() as string).toLowerCase()
        const token = await gateway.tokenAddressOf(tokenId)
        const metadata = await assetErc20Metadata(ethereum, token, gatewayAddress)
        let metadataOnAH: any, assetId: any
        if (locationOnAH?.parents == 0) {
            const assetId = locationOnAH?.interior?.x2[1]?.generalIndex
            metadataOnAH = (await assethub.query.assets.asset(assetId)).toJSON()
            metadataOnAH.assetId = assetId.toString()
        } else {
            if (
                locationOnAH?.parents == DOT_LOCATION.parents &&
                locationOnAH?.interior == DOT_LOCATION.interior
            ) {
                let existentialDeposit = assethub.consts.balances.existentialDeposit.toPrimitive()
                metadataOnAH = {
                    minBalance: existentialDeposit,
                    isSufficient: true,
                }
            } else {
                const assetType = assethub.registry.createType("StagingXcmV4Location", locationOnAH)
                metadataOnAH = (await assethub.query.foreignAssets.asset(assetType)).toJSON()
            }
        }
        const assetInfo: Asset = {
            token,
            name: metadata.name,
            symbol: metadata.symbol,
            decimals: metadata.decimals,
            locationOnEthereum: location,
            location: locationOnAH,
            locationOnAH,
            foreignId: tokenId,
            minimumBalance: metadataOnAH?.minBalance as bigint,
            isSufficient: metadataOnAH?.isSufficient as boolean,
            assetId: metadataOnAH?.assetId,
        }
        pnas.push(assetInfo)
    }
    let assetOverrides: any = {}
    assetOverrides[assetHubParaId.toString()] = pnas
    return assetOverrides
}

async function indexKusamaPNAs(
    bridgehub: ApiPromise,
    polkadotAssethub: ApiPromise,
    kusamaAssethub: ApiPromise,
    ethereum: AbstractProvider,
    gatewayAddress: string,
    assetHubParaId: number
): Promise<AssetOverrideMap> {
    let pnas: Asset[] = []
    let gateway = IGateway__factory.connect(gatewayAddress, ethereum)
    const entries = await bridgehub.query.ethereumSystem.nativeToForeignId.entries()
    for (const [key, value] of entries) {
        const location: any = key.args.at(0)?.toJSON()
        if (!location) {
            console.warn(`Could not convert ${key.toHuman()} to location`)
            continue
        }

        const locationOnAHKusama: any = bridgeablePNAsOnKusamaAH(location, assetHubParaId)
        const locationOnAHPolkadot: any = bridgeablePNAsOnPolkadotAH("", location, assetHubParaId)
        if (!locationOnAHKusama) {
            continue
        }
        // Check if asset is registered on Kusama Assethub, if is not native KSM
        if (
            locationOnAHKusama?.parents != DOT_LOCATION.parents &&
            locationOnAHKusama?.interior != DOT_LOCATION.interior
        ) {
            const assetType = kusamaAssethub.registry.createType(
                "StagingXcmV4Location",
                locationOnAHKusama
            )
            let assetOnKusama = (await kusamaAssethub.query.foreignAssets.asset(assetType)).toJSON()
            if (!assetOnKusama) {
                console.warn(
                    `Location ${JSON.stringify(
                        locationOnAHKusama
                    )} is not a registered asset on Kusama Assethub`
                )
                continue
            }
        }

        const tokenId = (value.toPrimitive() as string).toLowerCase()
        const token = await gateway.tokenAddressOf(tokenId)
        const metadata = await assetErc20Metadata(ethereum, token, gatewayAddress)
        let metadataOnAH: any, assetId: any
        if (locationOnAHKusama?.parents == 0) {
            // skip any Kusama native assets for now
            continue
        } else {
            if (
                locationOnAHKusama?.parents == DOT_LOCATION.parents &&
                locationOnAHKusama?.interior == DOT_LOCATION.interior
            ) {
                let existentialDeposit =
                    kusamaAssethub.consts.balances.existentialDeposit.toPrimitive()
                metadataOnAH = {
                    minBalance: existentialDeposit,
                    isSufficient: true,
                }
            } else if (
                locationOnAHPolkadot?.parents == DOT_LOCATION.parents &&
                locationOnAHPolkadot?.interior == DOT_LOCATION.interior
            ) {
                let existentialDeposit =
                    polkadotAssethub.consts.balances.existentialDeposit.toPrimitive()
                metadataOnAH = {
                    minBalance: existentialDeposit,
                    isSufficient: true,
                }
            }
        }
        const assetInfo: Asset = {
            token,
            name: metadata.name,
            symbol: metadata.symbol,
            decimals: metadata.decimals,
            locationOnEthereum: location,
            location: locationOnAHKusama,
            locationOnAH: locationOnAHKusama,
            foreignId: tokenId,
            minimumBalance: metadataOnAH?.minBalance as bigint,
            isSufficient: metadataOnAH?.isSufficient as boolean,
            assetId: metadataOnAH?.assetId,
        }
        pnas.push(assetInfo)
    }
    let assetOverrides: any = {}
    assetOverrides[assetHubParaId.toString()] = pnas
    return assetOverrides
}

export const WESTEND_GENESIS = "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e"

// Currently, the bridgeable assets are limited to KSM, DOT, native assets on AH
// and TEER
function bridgeablePNAsOnPolkadotAH(
    environment: string,
    location: any,
    assetHubParaId: number
): any {
    if (location.parents != 1) {
        return
    }
    // KSM
    if (location.interior.x1 && location.interior.x1[0]?.globalConsensus?.kusama !== undefined) {
        return {
            parents: 2,
            interior: {
                x1: [
                    {
                        globalConsensus: {
                            kusama: null,
                        },
                    },
                ],
            },
        }
    }
    // DOT
    else if (
        location.interior.x1 &&
        location.interior.x1[0]?.globalConsensus?.polkadot !== undefined
    ) {
        return DOT_LOCATION
    }
    // Native assets from AH
    else if (
        location.interior.x4 &&
        location.interior.x4[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x4[1]?.parachain == assetHubParaId
    ) {
        return {
            parents: 0,
            interior: {
                x2: [
                    {
                        palletInstance: location.interior.x4[2]?.palletInstance,
                    },
                    {
                        generalIndex: location.interior.x4[3]?.generalIndex,
                    },
                ],
            },
        }
    }
    // Others from 3rd Parachains, only TEER for now
    else if (
        location.interior.x2 &&
        location.interior.x2[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x2[1]?.parachain == 2039
    ) {
        return {
            parents: 1,
            interior: {
                x1: [
                    {
                        parachain: 2039,
                    },
                ],
            },
        }
    }
    // Add assets for Westend
    switch (environment) {
        case "westend_sepolia": {
            if (
                location.interior.x1 &&
                location.interior.x1[0]?.globalConsensus?.byGenesis === WESTEND_GENESIS
            ) {
                return DOT_LOCATION
            } else if (
                location.interior.x2 &&
                location.interior.x2[0]?.globalConsensus?.byGenesis === WESTEND_GENESIS &&
                location.interior.x2[1]?.parachain != undefined
            ) {
                return {
                    parents: 1,
                    interior: {
                        x1: [
                            {
                                parachain: location.interior.x2[1]?.parachain,
                            },
                        ],
                    },
                }
            }
        }
    }
}

export async function getAssetHubConversationPalletSwap(
    assetHub: ApiPromise,
    asset1: any,
    asset2: any,
    exactAsset2Balance: bigint
) {
    const result = await assetHub.call.assetConversionApi.quotePriceTokensForExactTokens(
        asset1,
        asset2,
        exactAsset2Balance,
        true
    )
    const asset1Balance = result.toPrimitive() as any
    if (asset1Balance == null) {
        throw Error(
            `No pool set up in asset conversion pallet for '${JSON.stringify(
                asset1
            )}' and '${JSON.stringify(asset2)}'.`
        )
    }
    return BigInt(asset1Balance)
}

function bridgeablePNAsOnKusamaAH(location: any, assetHubParaId: number): any {
    if (location.parents != 1) {
        return
    }
    // KSM
    if (location.interior.x1 && location.interior.x1[0]?.globalConsensus?.kusama !== undefined) {
        console.log("KSM")
        return {
            parents: 1,
            interior: "Here",
        }
    }
    // DOT
    else if (
        location.interior.x1 &&
        location.interior.x1[0]?.globalConsensus?.polkadot !== undefined
    ) {
        return {
            parents: 2,
            interior: {
                x1: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                ],
            },
        }
    }
    // Native assets from AH
    else if (
        location.interior.x4 &&
        location.interior.x4[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x4[1]?.parachain == assetHubParaId
    ) {
        return {
            parents: 2,
            interior: {
                x4: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                    {
                        parachain: assetHubParaId,
                    },
                    {
                        palletInstance: location.interior.x4[2]?.palletInstance,
                    },
                    {
                        generalIndex: location.interior.x4[3]?.generalIndex,
                    },
                ],
            },
        }
    }
    // Others from 3rd Parachains, only TEER for now
    else if (
        location.interior.x2 &&
        location.interior.x2[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x2[1]?.parachain == 2039
    ) {
        return {
            parents: 2,
            interior: {
                x2: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                    {
                        parachain: 2039,
                    },
                ],
            },
        }
    }
}
