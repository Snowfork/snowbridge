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
    BridgeInfo,
    TransferRoute,
    ChainId,
    ChainKey,
    ParachainKind,
} from "@snowbridge/base-types"
import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api"
import { isFunction } from "@polkadot/util"
import { writeFile } from "fs/promises"
import { AbstractProvider, Contract, ethers } from "ethers"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { parachains as ParaImpl, xcmBuilder, assetsV2 } from "@snowbridge/api"

export type Path = {
    source: ChainId
    destination: ChainId
    asset: string
}

const SNOWBRIDGE_ENV: { [env: string]: Environment } = {
    local_e2e: {
        name: "local_e2e",
        ethChainId: 11155111,
        beaconApiUrl: "http://127.0.0.1:9596",
        ethereumChains: {
            "11155111": "ws://127.0.0.1:8546",
        },
        relaychainUrl: "ws://127.0.0.1:9944",
        parachains: {
            "1000": "ws://127.0.0.1:12144",
            "1002": "ws://127.0.0.1:11144",
            "2000": "ws://127.0.0.1:13144",
        },
        gatewayContract: "0xb1185ede04202fe62d38f5db72f71e38ff3e8305",
        beefyContract: "0x83428c7db9815f482a39a1715684dcf755021997",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        v2_parachains: [1000],
        indexerGraphQlUrl: "http://127.0.0.1/does/not/exist",
    },
    paseo_sepolia: {
        name: "paseo_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://paseo-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-paseo-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-paseo.dotters.network",
            "3369": "wss://paseo-muse-rpc.polkadot.io",
            "2043": `wss://parachain-testnet-rpc.origin-trail.network`,
        },
        gatewayContract: "0x1607c1368bc943130258318c91bbd8cff3d063e6",
        beefyContract: "0x2c780945beb1241fe9c645800110cb9c4bbbb639",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        v2_parachains: [1000],
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-paseo@v1/api/graphql",
        metadataOverrides: {
            // Change the name of TRAC
            "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                name: "OriginTrail TRAC",
                symbol: "TRAC",
            },
        },
    },
    polkadot_mainnet: {
        name: "polkadot_mainnet",
        ethChainId: 1,
        beaconApiUrl: "https://lodestar-mainnet.chainsafe.io",
        ethereumChains: {
            "1": "https://ethereum-rpc.publicnode.com",
            "1284": "https://rpc.api.moonbeam.network",
            "8453": "https://base-rpc.publicnode.com",
        },
        relaychainUrl: "https://polkadot-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-polkadot-rpc.n.dwellir.com",
            "1002": "https://bridge-hub-polkadot-rpc.n.dwellir.com",
            "3369": "wss://polkadot-mythos-rpc.polkadot.io",
            "2034": "wss://hydration-rpc.n.dwellir.com",
            "2030": "wss://bifrost-polkadot.ibp.network",
            "2004": "wss://moonbeam.ibp.network",
            "2000": "wss://acala-rpc-0.aca-api.network",
            "2043": "wss://parachain-rpc.origin-trail.network",
            // TODO: Add back in jampton once we have an indexer in place.
            //"3397": "wss://rpc.jamton.network",
        },
        gatewayContract: "0x27ca963c279c93801941e1eb8799c23f407d68e7",
        beefyContract: "0x1817874feab3ce053d0f40abc23870db35c2affc",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        v2_parachains: [1000],
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-polkadot@v2/api/graphql",
        kusama: {
            assetHubParaId: 1000,
            bridgeHubParaId: 1002,
            parachains: {
                "1000": "wss://asset-hub-kusama-rpc.n.dwellir.com",
                "1002": "https://bridge-hub-kusama-rpc.n.dwellir.com",
            },
        },
        precompiles: {
            // Add override for mythos token and add precompile for moonbeam
            "2004": "0x000000000000000000000000000000000000081a",
        },
        metadataOverrides: {
            // Change the name of TRAC
            "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f": {
                name: "OriginTrail TRAC",
            },
        },
        l2Bridge: {
            acrossAPIUrl: "https://app.across.to/api",
            l1AdapterAddress: "0x313e8c9fb47613f2b1a436be978c2bb75727fcc5",
            l1HandlerAddress: "0x924a9f036260ddd5808007e1aa95f08ed08aa569",
            l1FeeTokenAddress: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            l1SwapQuoterAddress: "0x61ffe014ba17989e743c5f6cb21bf9697530b21e",
            l1SwapRouterAddress: "0xe592427a0aece92de3edee1f18e0157c05861564",
            l2Chains: {
                "8453": {
                    adapterAddress: "0xcd5d2c665e3ac84bf5c67fe7a0c48748da40db2f",
                    feeTokenAddress: "0x4200000000000000000000000000000000000006",
                    swapRoutes: [
                        // WETH
                        {
                            inputToken: "0x4200000000000000000000000000000000000006",
                            outputToken: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                            swapFee: 0,
                        },
                        // USDC
                        {
                            inputToken: "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
                            outputToken: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                            swapFee: 500,
                        },
                    ],
                },
            },
        },
    },
    westend_sepolia: {
        name: "westend_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
            "84532": "https://base-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://westend-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-westend-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-westend-rpc.n.dwellir.com",
        },
        gatewayContract: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
        beefyContract: "0xa04460b1d8bbef33f54edb2c3115e3e4d41237a6",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        v2_parachains: [1000],
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-westend@v1/api/graphql",
        l2Bridge: {
            acrossAPIUrl: "https://testnet.across.to/api",
            l1AdapterAddress: "0xa5b8589bd534701be49916c4d2e634ab1c765cbf",
            l1HandlerAddress: "0x924a9f036260ddd5808007e1aa95f08ed08aa569",
            l1FeeTokenAddress: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
            l1SwapRouterAddress: "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e",
            l1SwapQuoterAddress: "0xed1f6473345f45b75f8179591dd5ba1888cf2fb3",
            l2Chains: {
                "84532": {
                    adapterAddress: "0xf06939613a3838af11104c898758220db9093679",
                    feeTokenAddress: "0x4200000000000000000000000000000000000006",
                    swapRoutes: [
                        // WETH
                        {
                            inputToken: "0x4200000000000000000000000000000000000006",
                            outputToken: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                            swapFee: 0,
                        },
                        // USDC
                        {
                            inputToken: "0x036cbd53842c5426634e7929541ec2318f3dcf7e",
                            outputToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                            swapFee: 500,
                        },
                    ],
                },
            },
        },
    },
}

export function defaultPathFilter(envName: string): (_: Path) => boolean {
    switch (envName) {
        case "westend_sepolia": {
            return (path: Path) => {
                // Frequency
                if (path.asset === "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620") {
                    return false
                }
                // Disable para to para transfers
                if (path.source.kind === "polkadot" && path.destination.kind === "polkadot") {
                    return false
                }
                return true
            }
        }
        case "paseo_sepolia":
            return (path: Path) => {
                // Disallow MUSE to any location but 3369
                if (
                    path.asset === "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee" &&
                    ((path.destination.kind === "polkadot" &&
                        path.destination.id !== 3369 &&
                        path.source.kind === "ethereum") ||
                        (path.source.id !== 3369 && path.source.kind === "polkadot"))
                ) {
                    return false
                }
                // Disable para to para transfers
                if (path.source.kind === "polkadot" && path.destination.kind === "polkadot") {
                    return false
                }
                return true
            }
        case "polkadot_mainnet":
            return (path: Path) => {
                // Disallow MYTH to any location but 3369
                if (
                    path.asset === "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003" &&
                    ((path.destination.kind === "polkadot" &&
                        path.destination.id !== 3369 &&
                        path.source.kind === "ethereum") ||
                        (path.source.id !== 3369 && path.source.kind === "polkadot"))
                ) {
                    return false
                }

                // Allow TRAC to go to Hydration (2034) and Neuroweb (2043) only
                if (
                    path.asset === "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f" &&
                    ((path.destination.kind === "polkadot" &&
                        path.destination.id !== 2034 &&
                        path.destination.id !== 2043 &&
                        path.source.kind === "ethereum") ||
                        (path.source.id !== 2034 &&
                            path.source.id !== 2043 &&
                            path.source.kind === "polkadot"))
                ) {
                    return false
                }

                // Disable stable coins in the UI from Ethereum to Polkadot
                if (
                    (path.asset === "0x9d39a5de30e57443bff2a8307a4256c8797a3497" || // Staked USDe
                        path.asset === "0xa3931d71877c0e7a3148cb7eb4463524fec27fbd" || // Savings USD
                        path.asset === "0x6b175474e89094c44da98b954eedeac495271d0f") && // DAI
                    path.destination.kind === "polkadot" &&
                    path.destination.id === 2034 // Hydration
                ) {
                    return false
                }
                // Disable para to para transfers except for hydration
                if (
                    path.source.kind === "polkadot" &&
                    path.destination.kind === "polkadot" &&
                    !(
                        (path.source.id === 2034 && path.destination.id == 1000) ||
                        (path.source.id === 1000 && path.destination.id === 2034)
                    )
                ) {
                    return false
                }
                return true
            }

        default:
            return (_: Path) => true
    }
}

function buildTransferLocations(
    registry: AssetRegistry,
    environment: Environment,
    filter?: (path: Path) => boolean,
): TransferRoute[] {
    const ethChain = registry.ethereumChains[`ethereum_${registry.ethChainId}`]
    const parachains = Object.values(registry.parachains).filter(
        (p) => !(p.kind === "polkadot" && p.id === registry.bridgeHubParaId),
    )

    const pathFilter = filter ?? defaultPathFilter(registry.environment)

    const locations: Path[] = []

    const ethAssets = Object.keys(ethChain.assets)
    // Bridged paths
    for (const parachain of parachains) {
        const destinationAssets = Object.keys(parachain.assets)
        const commonAssets = new Set(
            ethAssets.filter((sa) => destinationAssets.find((da) => da === sa)),
        )
        for (const asset of commonAssets) {
            const p1: Path = {
                source: { kind: ethChain.kind, id: ethChain.id },
                destination: { kind: parachain.kind, id: parachain.id },
                asset,
            }
            if (pathFilter(p1)) {
                locations.push(p1)
            }
            const p2: Path = {
                source: p1.destination,
                destination: p1.source,
                asset,
            }
            if (pathFilter(p2)) {
                locations.push(p2)
            }
            if (
                parachain.info.evmChainId &&
                registry.ethereumChains[`ethereum_${parachain.info.evmChainId}`]
            ) {
                const p3: Path = {
                    source: {
                        kind: "ethereum",
                        id: parachain.info.evmChainId,
                    },
                    destination: p1.source, // Ethereum
                    asset,
                }
                if (pathFilter(p3)) {
                    locations.push(p3)
                }
            }
        }
    }

    // Local paths
    const assetHub = registry.parachains[`polkadot_${registry.assetHubParaId}`]
    for (const parachain of parachains) {
        if (parachain.kind === assetHub.kind && parachain.id === assetHub.id) continue
        const assetHubAssets = Object.keys(assetHub.assets)
        const destinationAssets = Object.keys(parachain.assets)

        // The asset exists on ethereum, parachain and asset hub
        const commonAssets = new Set(
            ethAssets.filter(
                (sa) =>
                    assetHubAssets.find((da) => da === sa) &&
                    destinationAssets.find((da) => da === sa),
            ),
        )
        for (const asset of commonAssets) {
            const p1: Path = {
                source: { kind: assetHub.kind, id: assetHub.id },
                destination: { kind: parachain.kind, id: parachain.id },
                asset,
            }
            if (pathFilter(p1)) {
                locations.push(p1)
            }
            const p2: Path = {
                source: p1.destination, // Parachain
                destination: p1.source, // Asset Hub
                asset,
            }
            if (pathFilter(p2)) {
                locations.push(p2)
            }
        }
    }

    // L2 paths
    if (environment.l2Bridge) {
        // Do asset hub only, in future we can loop through all v2 enabled parachains.
        for (const l2ChainKey of Object.keys(environment.l2Bridge.l2Chains)) {
            const l2ChainId = Number(l2ChainKey)
            const l2Chain = environment.l2Bridge.l2Chains[l2ChainId]
            const ethChain = registry.ethereumChains[`ethereum_l2_${l2ChainId}`]
            if (!ethChain || !l2Chain) {
                console.warn(`Could not find ethereum l2 chain ${l2ChainId}. Skipping...`)
                continue
            }
            const assetHubAssets = Object.keys(assetHub.assets)
            const destinationAssets = Object.values(ethChain.assets)
                .map((a) => a.swapTokenAddress?.toLowerCase())
                .filter((a) => a !== undefined)

            // The asset exists on ethereum, parachain and asset hub
            const commonAssets = new Set(
                ethAssets.filter(
                    (sa) =>
                        assetHubAssets.find((da) => da === sa) &&
                        destinationAssets.find((da) => da === sa),
                ),
            )
            for (const asset of commonAssets) {
                const p1: Path = {
                    source: { kind: assetHub.kind, id: assetHub.id },
                    destination: { kind: ethChain.kind, id: ethChain.id },
                    asset,
                }
                if (pathFilter(p1)) {
                    locations.push(p1)
                }
                const p2: Path = {
                    source: p1.destination, // L2 Chain
                    destination: p1.source, // Asset Hub
                    asset,
                }
                if (pathFilter(p2)) {
                    locations.push(p2)
                }
            }
        }

        const results: TransferRoute[] = []
        for (const location of locations) {
            let source = results.find(
                (s) =>
                    s.from.kind === location.source.kind &&
                    s.from.id === location.source.id &&
                    s.to.kind === location.destination.kind &&
                    s.to.id === location.destination.id,
            )

            if (!source) {
                source = {
                    from: location.source,
                    to: location.destination,
                    assets: [],
                }
                results.push(source)
            }
            source.assets = source.assets.concat(location.asset)
        }
    }

    // Combine all paths into routes
    const results: TransferRoute[] = []
    for (const location of locations) {
        let source = results.find(
            (s) =>
                s.from.kind === location.source.kind &&
                s.from.id === location.source.id &&
                s.to.kind === location.destination.kind &&
                s.to.id === location.destination.id,
        )

        if (!source) {
            source = {
                from: location.source,
                to: location.destination,
                assets: [],
            }
            results.push(source)
        }
        source.assets = source.assets.concat(location.asset)
    }
    return results
}

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
                    "polkadot",
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
        paras[`polkadot_${parachainId}`] = para
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
        ethChains[ethChainInfo.key] = ethChainInfo
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
            "kusama",
            ethChainId,
            accessor.parachainId,
            assetHubParaId,
            pnaAssets,
            assetOverrides ?? {},
        )

        const kusamaParas: ParachainMap = {}
        kusamaParas[para.key] = para

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
    kind: ParachainKind,
    ethChainId: number,
    parachainId: number,
    assetHubParaId: number,
    pnaAssets: PNAMap,
    assetOverrides: AssetOverrideMap,
    v2_parachains?: readonly number[],
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
            assetsV2.ETHER_TOKEN_ADDRESS,
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
        id: parachainId,
        kind,
        key: `${kind}_${parachainId}`,
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
    const name = networkName !== "unknown" ? networkName : undefined
    if (networkChainId == ethChainId) {
        // Asset Hub and get meta data
        const assetHub = parachains[`polkadot_${assetHubParaId}`]
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
            kind: "ethereum",
            id: networkChainId,
            name,
            assets,
            key: `ethereum_${networkChainId}`,
            baseDeliveryGas: 120_000n,
        }
    } else if (networkChainId in l2Chains) {
        const assets: ERC20MetadataMap = {}
        for (const route of l2Chains[networkChainId].swapRoutes) {
            let asset = await assetErc20Metadata(provider, route.inputToken)
            assets[route.inputToken.toLowerCase()] = {
                ...asset,
                swapTokenAddress: route.outputToken.toLowerCase(),
                swapFee: route.swapFee,
            }
        }
        assets[assetsV2.ETHER_TOKEN_ADDRESS] = {
            token: assetsV2.ETHER_TOKEN_ADDRESS,
            name: "Ether",
            symbol: "Ether",
            decimals: 18,
            swapTokenAddress: assetsV2.ETHER_TOKEN_ADDRESS,
            swapFee: 0,
        }
        return {
            kind: "ethereum_l2",
            id: networkChainId,
            name,
            assets,
            key: `ethereum_l2_${networkChainId}`,
        }
    } else {
        let evmParachainChain: Parachain | undefined
        for (const paraId in parachains) {
            const parachain = parachains[paraId as ChainKey<"polkadot">]
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
        const paraId = evmParachainChain.id.toString()
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
            kind: "ethereum",
            id: networkChainId,
            key: `ethereum_${networkChainId}`,
            name,
            evmParachainId: evmParachainChain.id,
            assets,
            precompile,
            xcDOT: evmParachainChain.xcDOT,
            xcTokenMap,
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
        token: token.toLowerCase(),
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
    if (!(env in SNOWBRIDGE_ENV)) {
        throw Error(`Unknown environment ${env}.`)
    }
    const environment = SNOWBRIDGE_ENV[env]
    const registry = await buildRegistry(environment)
    const routes = buildTransferLocations(registry, environment)
    const bridge: BridgeInfo = { environment, routes, registry }
    const json = generateTsObject(bridge, 4)
    const fileContents = `const registry = ${json} as const\nexport default registry\n`
    const filepath = `src/${env}_bridge_info.g.ts`
    await writeFile(filepath, fileContents)
})()

function generateTsObject(value: unknown, indentSize = 4): string {
    const indentUnit = " ".repeat(indentSize)
    const serialize = (val: unknown, depth: number): string | undefined => {
        if (val === null) return "null"
        if (val === undefined) return undefined
        if (typeof val === "function" || typeof val === "symbol") return undefined
        if (typeof val === "bigint") return `${val}n`
        if (typeof val === "string") return JSON.stringify(val)
        if (typeof val === "number" || typeof val === "boolean") return String(val)
        if (Array.isArray(val)) {
            if (val.length === 0) return "[]"
            const indent = indentUnit.repeat(depth + 1)
            const closingIndent = indentUnit.repeat(depth)
            const items = val
                .map((item) => {
                    const serialized = serialize(item, depth + 1)
                    return `${indent}${serialized ?? "null"}`
                })
                .join(",\n")
            return `[\n${items}\n${closingIndent}]`
        }
        if (typeof val === "object") {
            const obj = val as Record<string, unknown>
            const keys = Object.keys(obj)
            const indent = indentUnit.repeat(depth + 1)
            const closingIndent = indentUnit.repeat(depth)
            const items: string[] = []
            for (const key of keys) {
                const serialized = serialize(obj[key], depth + 1)
                if (serialized === undefined) continue
                const keyLiteral = /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key)
                    ? key
                    : JSON.stringify(key)
                items.push(`${indent}${keyLiteral}: ${serialized},`)
            }
            if (items.length === 0) return "{}"
            return `{\n${items.join("\n")}\n${closingIndent}}`
        }
        throw new Error(`Unsupported type in registry output: ${typeof val}`)
    }

    const serialized = serialize(value, 0)
    if (serialized === undefined) {
        throw new Error("Registry output is not serializable")
    }
    return serialized
}
