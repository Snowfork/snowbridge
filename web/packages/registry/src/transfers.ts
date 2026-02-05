import {
    AssetRegistry,
    TransferLocation,
    Source,
    ChainKey,
    ChainKind,
    ChainId,
    TransferRoute,
} from "@snowbridge/base-types"

export function getTransferLocation(registry: AssetRegistry, chain: ChainId): TransferLocation {
    let location: TransferLocation | null = null
    if (chain.kind === "kusama" && registry.kusama) {
        const parachain = registry.kusama.parachains[`${chain.kind}_${chain.id}`]
        location = {
            id: parachain.id,
            kind: parachain.kind,
            name: parachain.info.name,
            key: parachain.key,
            parachain,
        }
    } else if (chain.kind === "polkadot") {
        const parachain = registry.parachains[`${chain.kind}_${chain.id}`]
        location = {
            id: parachain.id,
            kind: parachain.kind,
            name: parachain.info.name,
            key: parachain.key,
            parachain,
        }
    } else if (chain.kind === "ethereum") {
        const ethChain = registry.ethereumChains[`${chain.kind}_${chain.id}`]
        if (!ethChain.evmParachainId) {
            location = {
                kind: ethChain.kind,
                id: ethChain.id,
                key: ethChain.key,
                name: "Ethereum",
                ethChain,
            }
        } else {
            const evmChain = registry.parachains[`polkadot_${ethChain.evmParachainId}`]
            location = {
                kind: ethChain.kind,
                id: ethChain.id,
                key: ethChain.key,
                name: `${evmChain.info.name} (EVM)`,
                ethChain,
                parachain: evmChain,
            }
        }
    }

    if (location === null) throw Error(`Unknown ${chain.kind} chain ${chain.id}.`)

    return location
}

export function getTransferLocations(routes: readonly TransferRoute[]): Source[] {
    let sources: Source[] = []
    for (const route of routes) {
        let source = sources.find((s) => s.id === route.from.id && s.kind === route.from.kind)
        if (!source) {
            source = {
                key: `${route.from.kind}_${route.from.id}`,
                ...route.from,
                destinations: {},
            }
            sources.push(source)
        }
        const destId: ChainKey<ChainKind> = `${route.to.kind}_${route.to.id}`
        let destination = source.destinations[destId]
        if (!destination) {
            source.destinations[destId] = {
                key: destId,
                ...route.to,
                assets: [...route.assets],
            }
        }
    }
    return sources
}
