import {
    AssetRegistry,
    Environment,
    EthereumChain,
    Parachain,
    Path,
    Source,
    SourceType,
    TransferLocation,
} from "@snowbridge/base-types"
import { environmentFor } from "./environment"
import { assetRegistryFor } from "./registry"

const cache: { [env: string]: Source[] } = {}
export function transferSourcesFor(
    env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {}),
): Source[] {
    if (env in cache) {
        return cache[env]
    }
    return getTransferLocations(assetRegistryFor(env))
}

export function getEthereumTransferLocation(
    registry: AssetRegistry,
    ethChain: EthereumChain,
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
    sourceKey: string,
): TransferLocation {
    if (sourceType === "ethereum") {
        return getEthereumTransferLocation(registry, registry.ethereumChains[sourceKey])
    } else {
        return getSubstrateTransferLocation(registry.parachains[sourceKey])
    }
}

export function getTransferLocationKusama(
    registry: AssetRegistry,
    network: string,
    parachainId: string,
): TransferLocation {
    if (network === "kusama" && registry.kusama) {
        return getSubstrateTransferLocation(registry.kusama?.parachains[parachainId])
    } else {
        return getSubstrateTransferLocation(registry.parachains[parachainId])
    }
}

export function getTransferLocations(
    registry: AssetRegistry,
    filter?: (path: Path) => boolean,
): Source[] {
    const ethChain = registry.ethereumChains[registry.ethChainId]
    const parachains = Object.keys(registry.parachains)
        .filter((p) => p !== registry.bridgeHubParaId.toString())
        .map((p) => registry.parachains[p])

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
                type: "ethereum",
                id: "ethereum",
                source: ethChain.chainId,
                destinationType: "substrate",
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
                destinationType: "ethereum",
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
                    destinationType: "ethereum",
                    destination: ethChain.chainId,
                    asset,
                }
                if (pathFilter(p3)) {
                    locations.push(p3)
                }
            }
        }
    }

    // Local paths
    const assetHub = registry.parachains[registry.assetHubParaId]
    for (const parachain of parachains) {
        if (parachain.parachainId === assetHub.parachainId) continue
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
                type: "substrate",
                id: assetHub.info.specName,
                source: assetHub.parachainId,
                destinationType: "substrate",
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
                destinationType: "substrate",
                destination: assetHub.parachainId,
                asset,
            }
            if (pathFilter(p2)) {
                locations.push(p2)
            }
        }
    }

    const results: Source[] = []
    for (const location of locations) {
        let source = results.find(
            (s) =>
                s.type === location.type &&
                s.id === location.id &&
                s.key === location.source.toString(),
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
        let destination: { type: SourceType; assets: string[] } =
            source.destinations[location.destination]
        if (!destination) {
            destination = { type: location.destinationType, assets: [] }
            source.destinations[location.destination] = destination
        }
        destination.assets.push(location.asset)
    }
    return results
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
                if (path.type === "substrate" && path.destinationType === "substrate") {
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
                    ((path.destination !== 3369 && path.type === "ethereum") ||
                        (path.source !== 3369 && path.type === "substrate"))
                ) {
                    return false
                }
                // Disable para to para transfers
                if (path.type === "substrate" && path.destinationType === "substrate") {
                    return false
                }
                return true
            }
        case "polkadot_mainnet":
            return (path: Path) => {
                // Disallow MYTH to any location but 3369
                if (
                    path.asset === "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003" &&
                    ((path.destination !== 3369 && path.type === "ethereum") ||
                        (path.source !== 3369 && path.type === "substrate"))
                ) {
                    return false
                }

                // Allow TRAC to go to Hydration (2034) and Neuroweb (2043) only
                if (
                    path.asset === "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f" &&
                    ((path.destination !== 2034 &&
                        path.destination !== 2043 &&
                        path.type === "ethereum") ||
                        (path.source !== 2034 && path.source !== 2043 && path.type === "substrate"))
                ) {
                    return false
                }

                // Disable stable coins in the UI from Ethereum to Polkadot
                if (
                    (path.asset === "0x9d39a5de30e57443bff2a8307a4256c8797a3497" || // Staked USDe
                        path.asset === "0xa3931d71877c0e7a3148cb7eb4463524fec27fbd" || // Savings USD
                        path.asset === "0x6b175474e89094c44da98b954eedeac495271d0f") && // DAI
                    path.destination === 2034 // Hydration
                ) {
                    return false
                }
                // Disable para to para transfers except for hydration
                if (
                    path.type === "substrate" &&
                    path.destinationType === "substrate" &&
                    !(
                        (path.source === 2034 && path.destination == 1000) ||
                        (path.source === 1000 && path.destination === 2034)
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
