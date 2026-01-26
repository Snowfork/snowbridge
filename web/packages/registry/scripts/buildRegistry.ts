import "dotenv/config"
import {
    AssetOverrideMap,
    AssetRegistry,
    ChainProperties,
    Environment,
    ERC20Metadata,
    ERC20MetadataMap,
    ERC20MetadataOverrideMap,
    EthereumChain,
    KusamaConfig,
    L2ForwardMetadata,
    Parachain,
    ParachainMap,
    PNAMap,
    PrecompileMap,
    XC20TokenMap,
    XcmVersion,
} from "@snowbridge/base-types"
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { isFunction } from "@polkadot/util"
import { writeFile } from "fs/promises"
import { environmentFor } from "../src"
import { AbstractProvider, Contract, ethers } from "ethers"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { parachains as ParaImpl, xcmBuilder, assetsV2 } from "@snowbridge/api"

async function buildRegistry(environment: Environment): Promise<AssetRegistry> {
    const {
        relaychainUrl,
        ethereumChains,
        ethChainId,
        assetHubParaId,
        bridgeHubParaId,
        v2_parachains,
        parachains,
        gatewayContract,
        assetOverrides,
        precompiles,
        metadataOverrides,
        kusama,
        name,
        l2Bridge,
    } = environment

    let relayInfo: ChainProperties
    {
        let provider = await ApiPromise.create({
            noInitWarn: true,
            provider: relaychainUrl.startsWith("http")
                ? new HttpProvider(relaychainUrl)
                : new WsProvider(relaychainUrl),
        })
        relayInfo = await (await ParaImpl.paraImplementation(provider)).chainProperties()

        await provider.disconnect()
    }

    // Connect to all eth connections
    const ethProviders: {
        [chainId: string]: {
            chainId: number
            provider: AbstractProvider
            name: string
        }
    } = {}
    {
        for (const result of await Promise.all(
            Object.keys(ethereumChains).map(async (ethChain) => {
                let provider = ethers.getDefaultProvider(ethereumChains[ethChain])
                const network = await provider.getNetwork()
                return { chainId: Number(network.chainId), provider, name: network.name }
            }),
        )) {
            ethProviders[result.chainId.toString()] = result
        }
        if (!(ethChainId.toString() in ethProviders)) {
            throw Error(`Cannot find ethereum chain ${ethChainId} in the list of ethereum chains.`)
        }
    }

    let pnaAssets: PNAMap = {}
    let bridgeHubInfo: ChainProperties
    {
        if (!(bridgeHubParaId.toString() in parachains)) {
            throw Error(`Cannot find bridge hub ${bridgeHubParaId} in the list of parachains.`)
        }
        const bridgeHubUrl = parachains[bridgeHubParaId.toString()]
        let provider = await ApiPromise.create({
            noInitWarn: true,
            provider: bridgeHubUrl.startsWith("http")
                ? new HttpProvider(bridgeHubUrl)
                : new WsProvider(bridgeHubUrl),
        })
        bridgeHubInfo = await (await ParaImpl.paraImplementation(provider)).chainProperties()
        pnaAssets = await getRegisteredPnas(
            provider,
            ethProviders[ethChainId].provider,
            gatewayContract,
        )

        await provider.disconnect()
    }

    // Connect to all substrate parachains.
    const providers: {
        [paraIdKey: string]: { parachainId: number; accessor: ParaImpl.ParachainBase }
    } = {}
    {
        for (const { parachainId, accessor } of await Promise.all(
            Object.keys(parachains).map(async (parachainId) => {
                const parachainUrl = parachains[parachainId]
                const provider = await ApiPromise.create({
                    noInitWarn: true,
                    provider: parachainUrl.startsWith("http")
                        ? new HttpProvider(parachainUrl)
                        : new WsProvider(parachainUrl),
                })
                const accessor = await ParaImpl.paraImplementation(provider)
                return { parachainId: accessor.parachainId, accessor }
            }),
        )) {
            providers[parachainId.toString()] = { parachainId, accessor }
        }
        if (!(assetHubParaId.toString() in providers)) {
            throw Error(
                `Could not resolve asset hub para id ${assetHubParaId} in the list of parachains provided.`,
            )
        }
    }

    // Index parachains
    const paras: ParachainMap = {}
    for (const { parachainId, para } of await Promise.all(
        Object.keys(providers)
            .filter((parachainIdKey) => parachainIdKey !== bridgeHubParaId.toString())
            .map(async (parachainIdKey) => {
                const { parachainId, accessor } = providers[parachainIdKey]
                const para = await indexParachain(
                    accessor,
                    providers[assetHubParaId.toString()].accessor,
                    ethChainId,
                    parachainId,
                    assetHubParaId,
                    pnaAssets,
                    assetOverrides ?? {},
                    v2_parachains,
                )
                return { parachainId, para }
            }),
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
                gatewayContract,
                assetHubParaId,
                paras,
                precompiles ?? {},
                metadataOverrides ?? {},
                l2Bridge?.l2Chains ?? {},
            )
        }),
    )) {
        ethChains[ethChainInfo.chainId.toString()] = ethChainInfo
    }

    let kusamaConfig: KusamaConfig | undefined
    if (kusama) {
        const assetHubUrl = kusama.parachains[kusama.assetHubParaId.toString()]
        let provider = await ApiPromise.create({
            noInitWarn: true,
            provider: assetHubUrl.startsWith("http")
                ? new HttpProvider(assetHubUrl)
                : new WsProvider(assetHubUrl),
        })
        const accessor = await ParaImpl.paraImplementation(provider)

        const para = await indexParachain(
            accessor,
            providers[assetHubParaId].accessor,
            ethChainId,
            accessor.parachainId,
            assetHubParaId,
            pnaAssets,
            assetOverrides ?? {},
        )

        const kusamaParas: ParachainMap = {}
        kusamaParas[para.parachainId] = para

        kusamaConfig = {
            parachains: kusamaParas,
            assetHubParaId: kusama.assetHubParaId,
            bridgeHubParaId: kusama.bridgeHubParaId,
        }

        await accessor.provider.disconnect()
    }
    // Dispose of all substrate connections
    await Promise.all(
        Object.keys(providers).map(
            async (parachainKey) => await providers[parachainKey].accessor.provider.disconnect(),
        ),
    )

    // Dispose all eth connections
    Object.keys(ethProviders).forEach((parachainKey) =>
        ethProviders[parachainKey].provider.destroy(),
    )

    return {
        timestamp: new Date().toISOString(),
        environment: name,
        ethChainId,
        gatewayAddress: gatewayContract,
        assetHubParaId,
        bridgeHubParaId,
        relaychain: relayInfo,
        bridgeHub: bridgeHubInfo,
        ethereumChains: ethChains,
        parachains: paras,
        kusama: kusamaConfig,
    }
}

async function checkSnowbridgeV2Support(
    parachain: ParaImpl.ParachainBase,
    ethChainId: number,
): Promise<{
    xcmVersion: XcmVersion
    supportsAliasOrigin: boolean
    hasEthBalance: boolean
}> {
    let supportsAliasOrigin = false
    let hasEthBalance = false
    let xcmVersion: XcmVersion

    try {
        const testXcm = parachain.provider.registry.createType("XcmVersionedXcm", {
            v5: [
                {
                    aliasOrigin: {
                        parents: 0,
                        interior: {
                            x1: [
                                {
                                    accountId32: {
                                        id: "0x0000000000000000000000000000000000000000000000000000000000000000",
                                    },
                                },
                            ],
                        },
                    },
                },
            ],
        })

        const weightResult = (
            await parachain.provider.call.xcmPaymentApi.queryXcmWeight(testXcm)
        ).toPrimitive() as any

        if (weightResult.ok) {
            const refTime = BigInt(weightResult.ok.refTime.toString())
            const MAX_REASONABLE_WEIGHT = 10n ** 15n
            // Check if AliasOrigin is supported. Often, the XCM instruction
            // weight is set to MAX to make it unusable
            supportsAliasOrigin = refTime < MAX_REASONABLE_WEIGHT

            const etherLocation = {
                parents: 2,
                interior: { x1: [{ GlobalConsensus: { Ethereum: { chain_id: ethChainId } } }] },
            }

            // Check if ether is supported as a fee asset
            const feeResult = (
                await parachain.provider.call.xcmPaymentApi.queryWeightToAssetFee(weightResult.ok, {
                    v5: etherLocation,
                })
            ).toPrimitive() as any

            if (feeResult.ok) {
                hasEthBalance = true
            }
        }

        xcmVersion = "v5"
    } catch {
        // If any call throws an error, XCM V5 is likely not supported.
        xcmVersion = "v4"
    }

    return { xcmVersion, supportsAliasOrigin, hasEthBalance }
}

async function indexParachain(
    parachain: ParaImpl.ParachainBase,
    assetHub: ParaImpl.ParachainBase,
    ethChainId: number,
    parachainId: number,
    assetHubParaId: number,
    pnaAssets: PNAMap,
    assetOverrides: AssetOverrideMap,
    v2_parachains?: number[],
): Promise<Parachain> {
    const info = await parachain.chainProperties()

    const assets = await parachain.getAssets(ethChainId, pnaAssets)
    const xcDOT = parachain.getXC20DOT()
    const parachainIdKey = parachainId.toString()
    if (parachainIdKey in assetOverrides) {
        for (const asset of assetOverrides[parachainIdKey]) {
            assets[asset.token.toLowerCase()] = asset
        }
    }

    if (Object.keys(assets).length === 0) {
        console.warn(
            `Cannot discover assets for ${info.specName} (parachain ${parachainId}). Please add a handler for that runtime or add overrides.`,
        )
    }

    const hasPalletXcm = isFunction(
        parachain.provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen,
    )
    const hasDryRunRpc = isFunction(parachain.provider.rpc.system?.dryRun)
    const hasDryRunApi =
        isFunction(parachain.provider.call.dryRunApi?.dryRunCall) &&
        isFunction(parachain.provider.call.dryRunApi?.dryRunXcm)
    const hasTxPaymentApi = isFunction(parachain.provider.call.transactionPaymentApi?.queryInfo)
    const hasXcmPaymentApi = isFunction(parachain.provider.call.xcmPaymentApi?.queryXcmWeight)

    const { xcmVersion, supportsAliasOrigin, hasEthBalance } = await checkSnowbridgeV2Support(
        parachain,
        ethChainId,
    )

    // test getting balances
    let hasDotBalance = true
    try {
        await parachain.getDotBalance(
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
        )
    } catch (err) {
        console.warn(`Spec ${info.specName} does not support dot ${err}`)
        hasDotBalance = false
    }

    await parachain.getNativeBalance(
        info.accountType === "AccountId32"
            ? "0x0000000000000000000000000000000000000000000000000000000000000000"
            : "0x0000000000000000000000000000000000000000",
    )

    let estimatedExecutionFeeDOT = 0n
    let estimatedDeliveryFeeDOT = 0n
    if (parachainId !== assetHubParaId) {
        const destinationXcm = xcmBuilder.buildParachainERC20ReceivedXcmOnDestination(
            parachain.provider.registry,
            ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        estimatedDeliveryFeeDOT = await assetHub.calculateDeliveryFeeInDOT(
            parachainId,
            destinationXcm,
        )
        estimatedExecutionFeeDOT = await parachain.calculateXcmFee(
            destinationXcm,
            xcmBuilder.DOT_LOCATION,
        )
    }
    return {
        parachainId,
        features: {
            hasPalletXcm,
            hasDryRunApi,
            hasTxPaymentApi,
            hasDryRunRpc,
            hasDotBalance,
            hasEthBalance,
            hasXcmPaymentApi,
            supportsAliasOrigin,
            xcmVersion,
            supportsV2: v2_parachains?.includes(parachainId) ?? false,
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
    precompiles: PrecompileMap,
    metadataOverrides: ERC20MetadataOverrideMap,
    l2Chains: { [l2ChainId: number]: L2ForwardMetadata },
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
                continue // Skip unregistered assets
            }
            if (token === assetsV2.ETHER_TOKEN_ADDRESS) {
                assets[token] = {
                    token: assetHub.assets[token].token,
                    name: assetHub.assets[token].name,
                    symbol: assetHub.assets[token].symbol,
                    decimals: assetHub.assets[token].decimals,
                }
            } else {
                const [asset, foreignId] = await Promise.all([
                    assetErc20Metadata(provider, token),
                    gateway.queryForeignTokenID(token),
                ])
                assets[token] = {
                    ...asset,
                    foreignId:
                        foreignId !=
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                            ? foreignId
                            : undefined,
                    // LDO gas from https://etherscan.io/tx/0x4e984250beacf693e7407c6cfdcb51229f6a549aa857d601db868b572ee2364b
                    // Other ERC20 token transfer on Ethereum typically ranges from 45,000 to 65,000 gas units; use 80_000 to leave a margin
                    deliveryGas: asset.symbol == "LDO" ? 150_000n : 80_000n,
                }
            }
            if (token in metadataOverrides) {
                const override = metadataOverrides[token]
                const asset = assets[token]
                if (override.name) {
                    asset.name = override.name
                }
                if (override.symbol) {
                    asset.symbol = override.symbol
                }
                if (override.decimals) {
                    asset.decimals = override.decimals
                }
            }
        }
        if ((await provider.getCode(gatewayAddress)) === undefined) {
            throw Error(
                `Could not fetch code for gateway address ${gatewayAddress} on ethereum chain ${networkChainId}.`,
            )
        }
        return {
            chainId: networkChainId,
            assets,
            id: id ?? `chain_${networkChainId}`,
            baseDeliveryGas: 120_000n,
        }
    } else if (networkChainId in l2Chains) {
        const assets: ERC20MetadataMap = {}
        for (const route of l2Chains[networkChainId].swapRoutes) {
            let asset = await assetErc20Metadata(provider, route.inputToken)
            assets[route.inputToken] = {
                ...asset,
                swapTokenAddress: route.outputToken,
                swapFee: route.swapFee,
            }
        }
        assets["0x0000000000000000000000000000000000000000"] = {
            token: "0x0000000000000000000000000000000000000000",
            name: "Ether",
            symbol: "Ether",
            decimals: 18,
            swapTokenAddress: "0x0000000000000000000000000000000000000000",
            swapFee: 0,
        }
        return {
            chainId: networkChainId,
            assets,
            id: id ?? `l2_${networkChainId}`,
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
                `No precompile configured for parachain ${paraId} (ethereum chain ${networkChainId}).`,
            )
        }
        const precompile = precompiles[paraId]
        if ((await provider.getCode(precompile)) === undefined) {
            throw Error(
                `Could not fetch code for ${precompile} on parachain ${paraId} (ethereum chain ${networkChainId}).`,
            )
        }
        if (!evmParachainChain.xcDOT) {
            throw Error(`Could not find DOT XC20 address for evm chain ${networkChainId}.`)
        }
        const xc20DOTAsset: ERC20Metadata = await assetErc20Metadata(
            provider,
            evmParachainChain.xcDOT,
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
    foreignId?: string,
): Promise<ERC20Metadata> {
    const erc20Metadata = new Contract(token, ERC20_METADATA_ABI, provider)
    const [name, symbol, decimals] = await Promise.all([
        erc20Metadata.name(),
        erc20Metadata.symbol(),
        erc20Metadata.decimals(),
    ])
    return {
        token,
        name: String(name),
        symbol: String(symbol),
        decimals: Number(decimals),
        foreignId: foreignId,
    }
}

async function getRegisteredPnas(
    bridgehub: ApiPromise,
    ethereum: AbstractProvider,
    gatewayAddress: string,
): Promise<PNAMap> {
    let gateway = IGateway__factory.connect(gatewayAddress, ethereum)
    const entries = await bridgehub.query.ethereumSystem.foreignToNativeId.entries()
    const pnas: { [token: string]: { token: string; foreignId: string; ethereumlocation: any } } =
        {}
    for (const [key, value] of entries) {
        const location: any = value.toPrimitive()
        if (!location) {
            console.warn(`Could not convert ${key.toHuman()} to location`)
            continue
        }
        const tokenId = (key.args[0]?.toPrimitive() as string).toLowerCase()
        const token = await gateway.tokenAddressOf(tokenId)
        pnas[token.toLowerCase()] = {
            token: token.toLowerCase(),
            ethereumlocation: location,
            foreignId: tokenId,
        }
    }
    return pnas
}

;(async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const registry = await buildRegistry(environmentFor(env))
    const json = JSON.stringify(
        registry,
        (key, value) => {
            if (typeof value === "bigint") {
                return `bigint:${value.toString()}`
            }
            return value
        },
        2,
    )

    const filepath = `src/${env}.registry.json`
    await writeFile(filepath, json)
})()
