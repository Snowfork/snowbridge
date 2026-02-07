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
    switch (chain.kind) {
        case "kusama": {
            if (!registry.kusama) throw Error(`Kusama not configured.`)
            const parachain = registry.kusama.parachains[`${chain.kind}_${chain.id}`]
            return {
                id: parachain.id,
                kind: parachain.kind,
                key: parachain.key,
                parachain,
            }
        }
        case "polkadot": {
            const parachain = registry.parachains[`${chain.kind}_${chain.id}`]
            return {
                id: parachain.id,
                kind: parachain.kind,
                key: parachain.key,
                parachain,
            }
        }
        case "ethereum": {
            const ethChain = registry.ethereumChains[`${chain.kind}_${chain.id}`]
            if (!ethChain.evmParachainId) {
                return {
                    kind: ethChain.kind,
                    id: ethChain.id,
                    key: ethChain.key,
                    ethChain,
                }
            } else {
                const evmChain = registry.parachains[`polkadot_${ethChain.evmParachainId}`]
                return {
                    kind: ethChain.kind,
                    id: ethChain.id,
                    key: ethChain.key,
                    ethChain,
                    parachain: evmChain,
                }
            }
        }
        case "ethereum_l2": {
            const ethChain = registry.ethereumChains[`${chain.kind}_${chain.id}`]
            return {
                kind: ethChain.kind,
                id: ethChain.id,
                key: ethChain.key,
                ethChain,
            }
        }
        default:
            throw Error(`Unknown ${chain.kind} chain ${chain.id}.`)
    }
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
