import { AbstractProvider, Contract, ethers, WeiPerEther } from "ethers";
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api";
import { isFunction } from '@polkadot/util';
import { SnowbridgeEnvironment } from "./environment";
import { Context } from "./index";

function isSnowbridgeAsset(location: any, chainId: number) {
    return location.parents === 2 && location.interior.x2 !== undefined && location.interior.x2[0].globalConsensus.ethereum.chainId === chainId
}

export type ERC20Metadata = {
    token: string
    name: string
    symbol: string
    decimals: number
}

async function assetErc20Metadata(
    provider: AbstractProvider,
    token: string
): Promise<ERC20Metadata> {
    const erc20MetadataABI = [
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
    const erc20Metadata = new Contract(token, erc20MetadataABI, provider)
    const [name, symbol, decimals] = await Promise.all([
        erc20Metadata.name(),
        erc20Metadata.symbol(),
        erc20Metadata.decimals(),
    ])
    return { token, name: String(name), symbol: String(symbol), decimals: Number(decimals) }
}

interface XC20TokenMap { [xc20: string]: string }

interface ERC20MetadataMap { [token: string]: ERC20Metadata }
export type EthereumChain = {
    chainId: number
    evmParachainId?: number
    assets: ERC20MetadataMap
    precompile?: `0x${string}`
    xcDOT?: string
    xcTokenMap?: XC20TokenMap
}

export type ChainProperties = {
    tokenSymbols: string
    tokenDecimals: number
    ss58Format: number
    isEthereum: boolean
    accountType: "AccountId20" | "AccountId32"
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
    destinationFeeInDOT: bigint
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

interface AssetMap { [token: string]: Asset }

interface ParachainMap { [paraId: string]: Parachain }

interface PrecompileMap { [chainId: string]: `0x${string}` }

interface AssetOverrideMap { [paraId: string]: Asset[] }

interface FeeOverrideMap { [paraId: string]: bigint }

export type RegistryOptions = {
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
    gatewayAddress: string;
    ethChainId: number;
    assetHubParaId: number;
    bridgeHubParaId: number;
    relaychain: ChainProperties;
    bridgeHub: ChainProperties;
    ethereumChains: {
        [chainId: string]: EthereumChain;
    };
    parachains: ParachainMap;
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

async function calculateDestinationFee(provider: ApiPromise, ethChainId: number, padFeePercentage?: bigint) {
    const destinationXcm = provider.createType('XcmVersionedXcm',
        {
            v4: [
                {
                    reserveAssetDeposited: [
                        {
                            id: { parents: 1, interior: "Here" },
                            fun: {
                                Fungible: "340282366920938463463374607431768211455",
                            },
                        },
                        {
                            id: {
                                parents: 2,
                                interior: {
                                    X2: [
                                        { GlobalConsensus: { Ethereum: { chain_id: ethChainId } } },
                                        { AccountKey20: { key: "0x0000000000000000000000000000000000000000" } },
                                    ],
                                },
                            },
                            fun: {
                                Fungible: "340282366920938463463374607431768211455",
                            },
                        }
                    ]
                },
                { clearOrigin: null },
                {
                    buyExecution: {
                        fees: {
                            id: { parents: 1, interior: "Here" },
                            fun: {
                                Fungible: "340282366920938463463374607431768211455",
                            },
                        },
                        weightLimit: "Unlimited",
                    }
                },
                {
                    depositAsset: {
                        assets: {
                            wild: {
                                allCounted: 2,
                            },
                        },
                        beneficiary: {
                            parents: 0,
                            interior: { x1: [{ accountId32: { id: "0x0000000000000000000000000000000000000000000000000000000000000000" } }] },
                        },
                    }
                },
                { setTopic: "0x0000000000000000000000000000000000000000000000000000000000000000", }
            ]
        })
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
    const result = BigInt(feeInDot.ok.toString())
    
    // return fee padded by 15% unless another percentage is specified
    return result * (100n+(padFeePercentage ?? 15n))/100n
}

async function indexParachain(
    provider: ApiPromise,
    ethChainId: number,
    assetHubParaId: number,
    assetOverrides: AssetOverrideMap,
    destinationFeeOverrides: FeeOverrideMap,
): Promise<Parachain> {
    const info = await chainProperties(provider)
    const assets: AssetMap = {}
    let xcDOT: string | undefined
    switch (info.specName) {
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
    const parachainEncoded = await provider.query.parachainInfo.parachainId();
    const parachainId = Number(parachainEncoded.toPrimitive())

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

    let destinationFeeInDOT = 0n
    if (parachainId !== assetHubParaId) {
        if (hasXcmPaymentApi) {
            destinationFeeInDOT = await calculateDestinationFee(provider, ethChainId)
        } else {
            if (!(parachainIdKey in destinationFeeOverrides)) {
                throw Error(`Parachain ${parachainId} cannot fetch the destination fee and needs a fee override.`)
            }
            destinationFeeInDOT = destinationFeeOverrides[parachainIdKey]
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
        destinationFeeInDOT,
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

    if (chainId == ethChainId) {
        // Asset Hub and get meta data
        const assetHub = parachains[assetHubParaId.toString()]
        const assets: ERC20MetadataMap = {}
        // TODO: Check if tokens are actually registered via gateway
        for (const token in assetHub.assets) {
            const asset = await assetErc20Metadata(provider, token)
            assets[token] = asset
        }
        if (await provider.getCode(gatewayAddress) === undefined) {
            throw Error(`Could not fetch code for gatway address ${gatewayAddress} on ethereum chain ${chainId}.`)
        }
        return {
            chainId, assets
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
        const xc20TokenMap: XC20TokenMap = {}
        const assets: ERC20MetadataMap = {}
        for (const token in evmParachainChain.assets) {
            const xc20 = evmParachainChain.assets[token].xc20
            if (!xc20) { continue }
            const asset = await assetErc20Metadata(provider, xc20.toLowerCase())
            xc20TokenMap[token] = xc20
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
            chainId, evmParachainId: evmParachainChain.parachainId, assets, precompile, xcDOT: evmParachainChain.xcDOT
        }
    }
}

export async function buildRegistry(options: RegistryOptions): Promise<AssetRegistry> {
    const {
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

    const paras: ParachainMap = {}
    for (const parachain of parachains) {
        let provider: ApiPromise;
        if (typeof parachain === "string") {
            provider = await ApiPromise.create({
                provider: parachain.startsWith("http") ? new HttpProvider(parachain) : new WsProvider(parachain),
            })
        } else {
            provider = parachain
        }
        const para = await indexParachain(provider, ethChainId, assetHubParaId, assetOverrides ?? {}, destinationFeeOverrides ?? {})
        paras[para.parachainId.toString()] = para;

        if (typeof parachain === "string") {
            await provider.disconnect()
        }
    }
    if (!(assetHubParaId.toString() in paras)) {
        throw Error(`Could not resolve asset hub para id ${assetHubParaId} in the list of parachains provided.`)
    }

    const ethChains: { [chainId: string]: EthereumChain } = {}
    for (const ethChain of ethchains) {
        let provider: AbstractProvider;
        if (typeof ethChain === "string") {
            provider = ethers.getDefaultProvider(ethChain)
        } else {
            provider = ethChain
        }

        const ethChainInfo = await indexEthChain(provider, ethChainId, gatewayAddress, assetHubParaId, paras, precompiles ?? {})
        ethChains[ethChainInfo.chainId.toString()] = ethChainInfo;

        if (typeof ethChain === "string") {
            provider.destroy()
        }
    }
    if (!(ethChainId in ethChains)) {
        throw Error(`Cannot find ethereum chain ${ethChainId} in the list of ethereum chains.`)
    }

    return {
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

export function fromEnvironment({ config, ethChainId }: SnowbridgeEnvironment): RegistryOptions {
    return {
        assetHubParaId: config.ASSET_HUB_PARAID,
        bridgeHubParaId: config.BRIDGE_HUB_PARAID,
        bridgeHub: config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()],
        relaychain: config.RELAY_CHAIN_URL,
        ethChainId,
        gatewayAddress: config.GATEWAY_CONTRACT,
        ethchains: [config.ETHEREUM_API(process.env.REACT_APP_INFURA_KEY ?? "")],
        parachains: Object.keys(config.PARACHAINS)
            .filter(paraId => paraId !== config.BRIDGE_HUB_PARAID.toString())
            .map(paraId => config.PARACHAINS[paraId]),
    }
}

export async function fromContext(context: Context): Promise<RegistryOptions> {
    const { assetHubParaId, bridgeHubParaId } = context.config.polkadot
    return {
        assetHubParaId,
        bridgeHubParaId,
        bridgeHub: await context.bridgeHub(),
        relaychain: await context.relaychain(),
        ethChainId: Number((await context.ethereum().getNetwork()).chainId),
        gatewayAddress: await context.gateway().getAddress(),
        ethchains: [context.ethereum()],
        parachains: await Promise.all(
            context.parachains()
                .filter(paraId => paraId !== context.config.polkadot.bridgeHubParaId)
                .map(paraId => context.parachain(paraId))
        ),
    }
}
