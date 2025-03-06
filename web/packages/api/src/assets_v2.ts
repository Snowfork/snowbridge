import { AbstractProvider, Contract, ethers } from "ethers";
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api";
import { isFunction } from '@polkadot/util';
import { SnowbridgeEnvironment } from "./environment";
import { Context } from "./index";
import { buildParachainERC20ReceivedXcmOnDestination, DOT_LOCATION, erc20Location } from "./xcmBuilder";
import { IGateway__factory } from "@snowbridge/contract-types";

export type ERC20Metadata = {
    token: string
    name: string
    symbol: string
    decimals: number
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
    },
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
    precompiles?: PrecompileMap
    assetOverrides?: AssetOverrideMap
    destinationFeeOverrides?: FeeOverrideMap
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
}

interface AssetMap { [token: string]: Asset }

interface ParachainMap { [paraId: string]: Parachain }

interface PrecompileMap { [chainId: string]: `0x${string}` }

interface AssetOverrideMap { [paraId: string]: Asset[] }

interface FeeOverrideMap { [paraId: string]: bigint }

interface XC20TokenMap { [xc20: string]: string }

interface ERC20MetadataMap { [token: string]: ERC20Metadata }

export type SourceType = "substrate" | "ethereum"

export type Path = {
    type: SourceType;
    id: string;
    source: number;
    destination: number;
    asset: string;
}

export type Source = {
    type: SourceType;
    id: string;
    key: string;
    destinations: { [destination: string]: string[] }
}

export type TransferLocation = {
    id: string;
    name: string;
    key: string;
    type: SourceType;
    parachain?: Parachain;
    ethChain?: EthereumChain;
};

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
        precompiles,
        assetOverrides,
        destinationFeeOverrides
    } = options

    let relayInfo: ChainProperties
    {
        let provider: ApiPromise;
        if (typeof relaychain === "string") {
            provider = await ApiPromise.create({
                provider: relaychain.startsWith("http") ? new HttpProvider(relaychain) : new WsProvider(relaychain),
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
        let provider: ApiPromise;
        if (typeof bridgeHub === "string") {
            provider = await ApiPromise.create({
                provider: bridgeHub.startsWith("http") ? new HttpProvider(bridgeHub) : new WsProvider(bridgeHub),
            })
        } else {
            provider = bridgeHub
        }

        bridgeHubInfo = await chainProperties(provider)

        if (typeof bridgeHub === "string") {
            await provider.disconnect()
        }
    }

    const providers: { [paraIdKey: string]: { parachainId: number, provider: ApiPromise, managed: boolean } } = {}
    for (const { parachainId, provider, managed } of await Promise.all(
        parachains.map(async parachain => {
            let provider: ApiPromise;
            let managed = false
            if (typeof parachain === "string") {
                provider = await ApiPromise.create({
                    provider: parachain.startsWith("http") ? new HttpProvider(parachain) : new WsProvider(parachain),
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

    const paras: ParachainMap = {}
    for (const { parachainId, para } of await Promise.all(
        Object.keys(providers).map(async parachainIdKey => {
            const { parachainId, provider } = providers[parachainIdKey]
            const para = await indexParachain(
                provider,
                providers[assetHubParaId.toString()].provider,
                ethChainId,
                parachainId,
                assetHubParaId,
                assetOverrides ?? {},
                destinationFeeOverrides ?? {})
            return { parachainId, para }
        })
    )) {
        paras[parachainId.toString()] = para;
    }

    await Promise.all(
        Object.keys(providers)
            .filter(parachainKey => providers[parachainKey].managed)
            .map(async parachainKey => await providers[parachainKey].provider.disconnect())
    )

    if (!(assetHubParaId.toString() in paras)) {
        throw Error(`Could not resolve asset hub para id ${assetHubParaId} in the list of parachains provided.`)
    }

    const ethChains: { [chainId: string]: EthereumChain } = {}
    for (const ethChainInfo of await Promise.all(
        ethchains.map(async ethChain => {
            let provider: AbstractProvider;
            if (typeof ethChain === "string") {
                provider = ethers.getDefaultProvider(ethChain)
            } else {
                provider = ethChain
            }
            const ethChainInfo = await indexEthChain(provider, ethChainId, gatewayAddress, assetHubParaId, paras, precompiles ?? {})

            if (typeof ethChain === "string") {
                provider.destroy()
            }
            return ethChainInfo
        })
    )) {
        ethChains[ethChainInfo.chainId.toString()] = ethChainInfo;
    }

    if (!(ethChainId in ethChains)) {
        throw Error(`Cannot find ethereum chain ${ethChainId} in the list of ethereum chains.`)
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
    }
}

export function getEthereumTransferLocation(registry: AssetRegistry, ethChain: EthereumChain): TransferLocation {
    if (!ethChain.evmParachainId) {
        return {
            id: "ethereum",
            name: "Ethereum",
            type: "ethereum",
            key: ethChain.chainId.toString(),
            ethChain,
        };
    } else {
        const evmChain = registry.parachains[ethChain.evmParachainId];
        return {
            id: ethChain.id,
            name: `${evmChain.info.name} (EVM)`,
            key: ethChain.chainId.toString(),
            type: "ethereum",
            ethChain,
            parachain: evmChain,
        };
    }
}

export function getSubstrateTransferLocation(parachain: Parachain): TransferLocation {
    return {
        id: parachain.info.specName,
        name: parachain.info.name,
        key: parachain.parachainId.toString(),
        type: "substrate",
        parachain,
    };
}

export function getTransferLocation(
    registry: AssetRegistry,
    sourceType: string,
    sourceKey: string,
): TransferLocation {
    if (sourceType === "ethereum") {
        return getEthereumTransferLocation(registry, registry.ethereumChains[sourceKey])
    } else {
        return getSubstrateTransferLocation(registry.parachains[sourceKey])
    }
}

export function getTransferLocations(registry: AssetRegistry, filter?: (path: Path) => boolean): Source[] {
    const ethChain = registry.ethereumChains[registry.ethChainId]
    const parachains = Object.keys(registry.parachains)
        .filter(p => p !== registry.bridgeHubParaId.toString())
        .map(p => registry.parachains[p])

    const pathFilter = filter ?? defaultPathFilter(registry.environment)

    const locations: Path[] = []
    for (const parachain of parachains) {
        const sourceAssets = Object.keys(ethChain.assets)
        const destinationAssets = Object.keys(parachain.assets)
        const commonAssets = new Set(sourceAssets.filter(sa => destinationAssets.find(da => da === sa)))
        for (const asset of commonAssets) {
            const p1: Path = { type: "ethereum", id: "ethereum", source: ethChain.chainId, destination: parachain.parachainId, asset }
            if (pathFilter(p1)) { locations.push(p1) }
            const p2: Path = { type: "substrate", id: parachain.info.specName, source: parachain.parachainId, destination: ethChain.chainId, asset }
            if (pathFilter(p2)) { locations.push(p2) }
            if(parachain.info.evmChainId && registry.ethereumChains[parachain.info.evmChainId]) {
                const p3: Path = { type: "ethereum", id: `${parachain.info.specName}_evm`, source: parachain.info.evmChainId, destination: ethChain.chainId, asset }
                if (pathFilter(p3)) { locations.push(p3) }
            }
        }
    }
    const results: Source[] = []
    for (const location of locations) {
        let source = results.find(s => s.type === location.type && s.id === location.id && s.key === location.source.toString())
        if (!source) {
            source = { type: location.type, id: location.id, key: location.source.toString(), destinations: {} }
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

export function fromEnvironment({ name, config, ethChainId }: SnowbridgeEnvironment, ethereumApiKey?: string): RegistryOptions {
    const result: RegistryOptions = {
        environment: name,
        assetHubParaId: config.ASSET_HUB_PARAID,
        bridgeHubParaId: config.BRIDGE_HUB_PARAID,
        bridgeHub: config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()],
        relaychain: config.RELAY_CHAIN_URL,
        ethChainId,
        gatewayAddress: config.GATEWAY_CONTRACT,
        ethchains: Object.values(config.ETHEREUM_CHAINS).map(x => x(ethereumApiKey ?? "")),
        parachains: Object.keys(config.PARACHAINS)
            .filter(paraId => paraId !== config.BRIDGE_HUB_PARAID.toString())
            .map(paraId => config.PARACHAINS[paraId]),
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
            context.parachains()
                .filter(paraId => paraId !== context.config.polkadot.bridgeHubParaId)
                .map(paraId => context.parachain(paraId))
        ),
    ])
    const result: RegistryOptions = {
        environment: context.config.environment,
        assetHubParaId,
        bridgeHubParaId,
        bridgeHub,
        relaychain,
        ethChainId: Number(network.chainId),
        gatewayAddress,
        ethchains: context.ethChains().map(ethChainId => context.ethChain(ethChainId)),
        parachains
    }
    addOverrides(context.config.environment, result)
    return result
}

export async function getNativeAccount(provider: ApiPromise, account: string): Promise<SubstrateAccount> {
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
        }
    }
}

export async function getNativeBalance(provider: ApiPromise, account: string): Promise<bigint> {
    const accountData = await getNativeAccount(provider, account)
    return accountData.data.free
}

export async function getLocationBalance(provider: ApiPromise, specName: string, location: any, account: string): Promise<bigint> {
    switch (specName) {
        case "basilisk":
        case "hydradx": {
            const assetId = (await provider.query.assetRegistry.locationAssets(location)).toPrimitive()
            if (!assetId) {
                throw Error(`DOT not registered for spec ${specName}.`)
            }
            const accountData = (await provider.query.tokens.accounts(account, assetId)).toPrimitive() as any
            return BigInt(accountData?.free ?? 0n)
        }
        case "penpal-parachain":
        case "asset-hub-paseo":
        case "westmint":
        case "statemint": {
            const assetId = location
            const accountData = (await provider.query.foreignAssets.account(assetId, account)).toPrimitive() as any
            return BigInt(accountData?.balance ?? 0n)
        }
        case "bifrost":
        case "bifrost_paseo":
        case "bifrost_polkadot": {
            const assetId = (await provider.query.assetRegistry.locationToCurrencyIds(location)).toPrimitive()
            if (!assetId) {
                throw Error(`DOT not registered for spec ${specName}.`)
            }
            const accountData = (await provider.query.tokens.accounts(account, assetId)).toPrimitive() as any
            return BigInt(accountData?.free ?? 0n)
        }
        case "mythos":
        case "muse": {
            console.warn(`${specName} does not support DOT, returning 0.`)
            return 0n
        }
        case "moonriver":
        case "moonbeam": {
            const assetId = (await provider.query.assetManager.assetTypeId({ xcm: location })).toPrimitive()
            if (!assetId) {
                throw Error(`DOT not registered for spec ${specName}.`)
            }
            const accountData = (await provider.query.assets.account(assetId, account)).toPrimitive() as any
            return BigInt(accountData?.balance ?? 0n)
        }
        default:
            throw Error(`Cannot get DOT balance for spec ${specName}.`)
    }
}

export function getDotBalance(provider: ApiPromise, specName: string, account: string): Promise<bigint> {
    switch (specName) {
        case "asset-hub-paseo":
        case "westmint":
        case "statemint": {
            return getNativeBalance(provider, account)
        }
        default:
            return getLocationBalance(provider, specName, DOT_LOCATION, account)
    }
}

export function getTokenBalance(provider: ApiPromise, specName: string, account: string, ethChainId: number, tokenAddress: string) {
    return getLocationBalance(provider, specName, erc20Location(ethChainId, tokenAddress), account)
}

export async function getParachainId(parachain: ApiPromise): Promise<number> {
    const sourceParachainEncoded = await parachain.query.parachainInfo.parachainId();
    return Number(sourceParachainEncoded.toPrimitive())
}

export async function calculateDestinationFee(provider: ApiPromise, destinationXcm: any) {
    const weight = (await provider.call.xcmPaymentApi.queryXcmWeight(destinationXcm)).toPrimitive() as any
    if (!weight.ok) {
        throw Error(`Can not query XCM Weight.`)
    }

    const feeInDot = (await provider.call.xcmPaymentApi.queryWeightToAssetFee(
        weight.ok,
        { v4: { parents: 1, interior: "Here" } }
    )).toPrimitive() as any
    if (!feeInDot.ok) {
        throw Error(`Can not convert weight to fee in DOT.`)
    }
    const executionFee = BigInt(feeInDot.ok.toString())

    return executionFee
}

export async function calculateDeliveryFee(provider: ApiPromise, parachainId: number, destinationXcm: any) {
    const result = (await provider.call.xcmPaymentApi.queryDeliveryFees(
        { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } },
        destinationXcm)).toPrimitive() as any
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
        console.info('Could not find DOT in result', result)
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
    const [properties, name] = await Promise.all([provider.rpc.system.properties(), provider.rpc.system.chain()])
    const tokenSymbols = properties.tokenSymbol.unwrapOrDefault().at(0)?.toString()
    const tokenDecimals = properties.tokenDecimals.unwrapOrDefault().at(0)?.toNumber()
    const isEthereum = properties.isEthereum.toPrimitive()
    const ss58Format = provider.consts.system.ss58Prefix.toPrimitive() as number ?? properties.ss58Format.unwrapOr(null)?.toNumber()
    const { specName, specVersion } = provider.consts.system.version.toJSON() as any
    const accountType = provider.registry.getDefinition("AccountId")

    let evmChainId: number | undefined;
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
                if (!isSnowbridgeAsset(location, ethChainId)) { continue }

                const assetId = Number(id.args.at(0)?.toString())
                const asset: any = (await provider.query.assetRegistry.assets(assetId)).toPrimitive()
                const token = String(location.interior.x2[1].accountKey20.key.toLowerCase())
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
        case "statemint": {
            const entries = await provider.query.foreignAssets.asset.entries()
            for (const [key, value] of entries) {
                const location: any = key.args.at(0)?.toJSON()
                if (!location) {
                    console.warn(`Could not convert ${key.toHuman()} to location for ${specName}.`)
                    continue
                }
                if (!isSnowbridgeAsset(location, ethChainId)) { continue }

                const asset: any = value.toJSON()
                const assetMetadata: any = (await provider.query.foreignAssets.metadata(location)).toPrimitive()
                const token = String(location.interior.x2[1].accountKey20.key.toLowerCase())
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
                if (!isSnowbridgeAsset(location, ethChainId)) { continue }

                const assetId: any = key.args.at(0)
                const asset: any = (await provider.query.assetRegistry.currencyMetadatas(assetId)).toPrimitive()
                const token = String(location.interior.x2[1].accountKey20.key.toLowerCase())
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
            const entries = await provider.query.assetManager.assetIdType.entries()
            for (const [key, value] of entries) {
                const location = (value.toJSON() as any).xcm

                const assetId = BigInt(key.args.at(0)?.toPrimitive() as any)
                const xc20 = assetId.toString(16).toLowerCase()

                if (location.parents === 1 && location.interior.here !== undefined) {
                    xcDOT = '0xffffffff' + xc20
                }

                if (!isSnowbridgeAsset(location, ethChainId)) { continue }

                const asset: any = (await provider.query.assets.asset(assetId)).toPrimitive()
                const metadata: any = (await provider.query.assets.metadata(assetId)).toPrimitive()
                const token = String(location.interior.x2[1].accountKey20.key.toLowerCase())
                assets[token] = {
                    token,
                    name: String(metadata.name),
                    minimumBalance: BigInt(asset.minBalance),
                    symbol: String(metadata.symbol),
                    decimals: Number(metadata.decimals),
                    isSufficient: Boolean(asset.isSufficient),
                    xc20: '0xffffffff' + xc20
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
    destinationFeeOverrides: FeeOverrideMap,
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
        throw Error(`Cannot discover assets for ${info.specName} (parachain ${parachainId}). Please add a handler for that runtime or add overrides.`)
    }

    const hasPalletXcm = isFunction(provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen);
    const hasDryRunRpc = isFunction(provider.rpc.system?.dryRun)
    const hasDryRunApi = isFunction(provider.call.dryRunApi?.dryRunCall) && isFunction(provider.call.dryRunApi?.dryRunXcm)
    const hasTxPaymentApi = isFunction(provider.call.transactionPaymentApi?.queryInfo)
    const hasXcmPaymentApi = isFunction(provider.call.xcmPaymentApi?.queryXcmWeight)
        && isFunction(provider.call.xcmPaymentApi?.queryDeliveryFees)
        && isFunction(provider.call.xcmPaymentApi?.queryWeightToAssetFee)

    if (info.accountType === "AccountId32") {
        await getDotBalance(provider, info.specName, "0x0000000000000000000000000000000000000000000000000000000000000000")
        await getNativeBalance(provider, "0x0000000000000000000000000000000000000000000000000000000000000000")
    } else {
        await getDotBalance(provider, info.specName, "0x0000000000000000000000000000000000000000")
        await getNativeBalance(provider, "0x0000000000000000000000000000000000000000")
    }

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
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        estimatedDeliveryFeeDOT = await calculateDeliveryFee(assetHub, parachainId, destinationXcm)
        if (hasXcmPaymentApi) {
            estimatedExecutionFeeDOT = await calculateDestinationFee(provider, destinationXcm)
        } else {
            if (!(parachainIdKey in destinationFeeOverrides)) {
                throw Error(`Parachain ${parachainId} cannot fetch the destination fee and needs a fee override.`)
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
            hasXcmPaymentApi
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
    ethChainId: number,
    gatewayAddress: string,
    assetHubParaId: number,
    parachains: ParachainMap,
    precompiles: PrecompileMap
): Promise<EthereumChain> {
    const network = await provider.getNetwork()
    const chainId = Number(network.chainId)
    const id = network.name !== "unknown" ? network.name : undefined

    if (chainId == ethChainId) {
        // Asset Hub and get meta data
        const assetHub = parachains[assetHubParaId.toString()]
        const gateway = IGateway__factory.connect(gatewayAddress, provider)

        const assets: ERC20MetadataMap = {}
        for (const token in assetHub.assets) {
            if (token === "0x0000000000000000000000000000000000000000") {
                // TODO: Support Native Ether
                continue;
            }
            if (!await gateway.isTokenRegistered(token)) {
                console.warn(`Token ${token} is not registered with the gateway.`)
            }
            const asset = await assetErc20Metadata(provider, token)
            assets[token] = asset
        }
        if (await provider.getCode(gatewayAddress) === undefined) {
            throw Error(`Could not fetch code for gatway address ${gatewayAddress} on ethereum chain ${chainId}.`)
        }
        return {
            chainId, assets, id: id ?? `chain_${chainId}`
        }
    } else {
        let evmParachainChain: Parachain | undefined;
        for (const paraId in parachains) {
            const parachain = parachains[paraId];
            if (parachain.info.evmChainId === chainId) {
                evmParachainChain = parachain;
                break
            }

        }
        if (!evmParachainChain) {
            throw Error(`Could not find evm chain ${chainId} in the list of parachains.`)
        }
        const xcTokenMap: XC20TokenMap = {}
        const assets: ERC20MetadataMap = {}
        for (const token in evmParachainChain.assets) {
            const xc20 = evmParachainChain.assets[token].xc20
            if (!xc20) { continue }
            const asset = await assetErc20Metadata(provider, xc20.toLowerCase())
            xcTokenMap[token] = xc20
            assets[xc20] = asset
        }
        const paraId = evmParachainChain.parachainId.toString()
        if (!(paraId in precompiles)) {
            throw Error(`No precompile configured for parachain ${paraId} (ethereum chain ${chainId}).`)
        }
        const precompile = precompiles[paraId]
        if (await provider.getCode(precompile) === undefined) {
            throw Error(`Could not fetch code for ${precompile} on parachain ${paraId} (ethereum chain ${chainId}).`)
        }
        if (!evmParachainChain.xcDOT) {
            throw Error(`Could not DOT XC20 address for evm chain ${chainId}.`)
        }
        const xc20DOTAsset: ERC20Metadata = await assetErc20Metadata(provider, evmParachainChain.xcDOT)
        assets[evmParachainChain.xcDOT] = xc20DOTAsset

        return {
            chainId,
            evmParachainId: evmParachainChain.parachainId,
            assets,
            precompile,
            xcDOT: evmParachainChain.xcDOT,
            xcTokenMap,
            id: id ?? `evm_${evmParachainChain.info.specName}`
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
    token: string
): Promise<ERC20Metadata> {
    const erc20Metadata = new Contract(token, ERC20_METADATA_ABI, provider)
    const [name, symbol, decimals] = await Promise.all([
        erc20Metadata.name(),
        erc20Metadata.symbol(),
        erc20Metadata.decimals(),
    ])
    return { token, name: String(name), symbol: String(symbol), decimals: Number(decimals) }
}

function isSnowbridgeAsset(location: any, chainId: number) {
    return location.parents === 2 && location.interior.x2 !== undefined && location.interior.x2[0].globalConsensus.ethereum.chainId === chainId
}

function addOverrides(envName: string, result: RegistryOptions) {
    switch (envName) {
        case "paseo_sepolia": {
            // Add override for mythos token and add precompile for moonbeam
            result.destinationFeeOverrides = {
                "3369": 200_000_000_000n
            }
            result.assetOverrides = {
                "3369": [
                    {
                        token: "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee".toLowerCase(),
                        name: "Muse",
                        minimumBalance: 10_000_000_000_000_000n,
                        symbol: "MUSE",
                        decimals: 18,
                        isSufficient: true,
                    }
                ]
            }
            break;
        }
        case "polkadot_mainnet": {
            // Add override for mythos token and add precompile for moonbeam
            result.precompiles = { "2004": "0x000000000000000000000000000000000000081a" }
            result.destinationFeeOverrides = {
                "3369": 100_000_000n
            }
            result.assetOverrides = {
                "3369": [
                    {
                        token: "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003".toLowerCase(),
                        name: "Mythos",
                        minimumBalance: 10_000_000_000_000_000n,
                        symbol: "MYTH",
                        decimals: 18,
                        isSufficient: true,
                    }
                ]
            }
            break;
        }
    }
}

function defaultPathFilter(envName: string): (_: Path) => boolean {
    switch (envName) {
        case "paseo_sepolia":
            return (path: Path) => {
                // Disallow MUSE to any location but 3369
                if (
                    path.asset === "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee" &&
                    path.destination !== 3369
                ) {
                    return false
                }
                return true
            }
        case "polkadot_mainnet":
            return (path: Path) => {

                // Disallow MYTH to any location but 3369
                if (
                    path.asset === "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003" &&
                    path.destination !== 3369
                ) {
                    return false
                }
                return true
            };

        default:
            return (_: Path) => true
    }
}
