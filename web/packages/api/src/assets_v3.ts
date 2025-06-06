import { AbstractProvider, Contract, ethers } from "ethers"
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { isFunction } from "@polkadot/util"
import { SnowbridgeEnvironment } from "./environment"
import { Context } from "./index"
import { buildParachainERC20ReceivedXcmOnDestination, DOT_LOCATION } from "./xcmV5Builder"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { MUSE_TOKEN_ID, MYTHOS_TOKEN_ID } from "./parachains/mythos"
import { paraImplementation } from "./parachains"
import { ParachainBase } from "./parachains/parachainBase"

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

type KusamaConfig = {
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

interface ParachainMap {
    [paraId: string]: Parachain
}

interface PrecompileMap {
    [chainId: string]: `0x${string}`
}

interface AssetOverrideMap {
    [paraId: string]: Asset[]
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
        assetOverrides,
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

        relayInfo = await (await paraImplementation(provider)).chainProperties()

        if (typeof relaychain === "string") {
            await provider.disconnect()
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

    let pnaAssets: PNAMap = {}
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
        bridgeHubInfo = await (await paraImplementation(provider)).chainProperties()
        pnaAssets = await getRegisteredPnas(
            provider,
            ethProviders[ethChainId].provider,
            gatewayAddress
        )

        if (typeof bridgeHub === "string") {
            await provider.disconnect()
        }
    }

    // Connect to all substrate parachains.
    const providers: {
        [paraIdKey: string]: { parachainId: number; accessor: ParachainBase; managed: boolean }
    } = {}
    {
        for (const { parachainId, accessor, managed } of await Promise.all(
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
                const accessor = await paraImplementation(provider)
                return { parachainId: accessor.parachainId, accessor, managed }
            })
        )) {
            providers[parachainId.toString()] = { parachainId, accessor, managed }
        }
        if (!(assetHubParaId.toString() in providers)) {
            throw Error(
                `Could not resolve asset hub para id ${assetHubParaId} in the list of parachains provided.`
            )
        }
    }

    // Index parachains
    const paras: ParachainMap = {}
    for (const { parachainId, para } of await Promise.all(
        Object.keys(providers).map(async (parachainIdKey) => {
            const { parachainId, accessor } = providers[parachainIdKey]
            const para = await indexParachain(
                accessor,
                providers[assetHubParaId.toString()].accessor,
                ethChainId,
                parachainId,
                assetHubParaId,
                pnaAssets,
                assetOverrides ?? {}
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

    let kusamaConfig: KusamaConfig | undefined
    if (kusama) {
        let provider: ApiPromise
        let managed = false
        if (typeof kusama.assetHub === "string") {
            provider = await ApiPromise.create({
                noInitWarn: true,
                provider: kusama.assetHub.startsWith("http")
                    ? new HttpProvider(kusama.assetHub)
                    : new WsProvider(kusama.assetHub),
            })
            managed = true
        } else {
            provider = kusama.assetHub
        }
        const accessor = await paraImplementation(provider)

        const para = await indexParachain(
            accessor,
            providers[assetHubParaId].accessor,
            ethChainId,
            accessor.parachainId,
            assetHubParaId,
            pnaAssets,
            assetOverrides ?? {}
        )

        const kusamaParas: ParachainMap = {}
        kusamaParas[kusama.assetHubParaId] = para

        kusamaConfig = {
            parachains: kusamaParas,
            assetHubParaId: kusama.assetHubParaId,
            bridgeHubParaId: kusama.bridgeHubParaId,
        }

        if (managed) {
            accessor.provider.disconnect()
        }
    }
    // Dispose of all substrate connections
    await Promise.all(
        Object.keys(providers)
            .filter((parachainKey) => providers[parachainKey].managed)
            .map(
                async (parachainKey) => await providers[parachainKey].accessor.provider.disconnect()
            )
    )

    // Dispose all eth connections
    Object.keys(ethProviders)
        .filter((parachainKey) => ethProviders[parachainKey].managed)
        .forEach((parachainKey) => ethProviders[parachainKey].provider.destroy())

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

export function padFeeByPercentage(fee: bigint, padPercent: bigint) {
    if (padPercent < 0 || padPercent > 100) {
        throw Error(`padPercent ${padPercent} not in range of 0 to 100.`)
    }
    return fee * ((100n + padPercent) / 100n)
}

async function indexParachain(
    parachain: ParachainBase,
    assetHub: ParachainBase,
    ethChainId: number,
    parachainId: number,
    assetHubParaId: number,
    pnaAssets: PNAMap,
    assetOverrides: AssetOverrideMap
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
            `Cannot discover assets for ${info.specName} (parachain ${parachainId}). Please add a handler for that runtime or add overrides.`
        )
    }

    const hasPalletXcm = isFunction(
        parachain.provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen
    )
    const hasDryRunRpc = isFunction(parachain.provider.rpc.system?.dryRun)
    const hasDryRunApi =
        isFunction(parachain.provider.call.dryRunApi?.dryRunCall) &&
        isFunction(parachain.provider.call.dryRunApi?.dryRunXcm)
    const hasTxPaymentApi = isFunction(parachain.provider.call.transactionPaymentApi?.queryInfo)

    // test getting balances
    let hasDotBalance = true
    try {
        await parachain.getDotBalance(
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000"
        )
    } catch (err) {
        console.warn(`Spec ${info.specName} does not support dot ${err}`)
        hasDotBalance = false
    }

    await parachain.getNativeBalance(
        info.accountType === "AccountId32"
            ? "0x0000000000000000000000000000000000000000000000000000000000000000"
            : "0x0000000000000000000000000000000000000000"
    )

    let estimatedExecutionFeeDOT = 0n
    let estimatedDeliveryFeeDOT = 0n
    if (parachainId !== assetHubParaId) {
        const destinationXcm = buildParachainERC20ReceivedXcmOnDestination(
            parachain.provider.registry,
            ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            info.accountType === "AccountId32"
                ? "0x0000000000000000000000000000000000000000000000000000000000000000"
                : "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        estimatedDeliveryFeeDOT = await assetHub.calculateDeliveryFeeInDOT(
            parachainId,
            destinationXcm
        )
        estimatedExecutionFeeDOT = await parachain.calculateXcmFee(destinationXcm, DOT_LOCATION)
    }
    return {
        parachainId,
        features: {
            hasPalletXcm,
            hasDryRunApi,
            hasTxPaymentApi,
            hasDryRunRpc,
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
                }
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
    foreignId?: string
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

function addOverrides(envName: string, result: RegistryOptions) {
    switch (envName) {
        case "polkadot_mainnet": {
            // Add override for mythos token and add precompile for moonbeam
            result.precompiles = { "2004": "0x000000000000000000000000000000000000081a" }
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
                    ((path.destination !== 3369 && path.type === "ethereum") ||
                        (path.source !== 3369 && path.type === "substrate"))
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

async function getRegisteredPnas(
    bridgehub: ApiPromise,
    ethereum: AbstractProvider,
    gatewayAddress: string
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
        const tokenId = (key.args.at(0)?.toPrimitive() as string).toLowerCase()
        const token = await gateway.tokenAddressOf(tokenId)
        pnas[token.toLowerCase()] = {
            token: token.toLowerCase(),
            ethereumlocation: location,
            foreignId: tokenId,
        }
    }
    return pnas
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
