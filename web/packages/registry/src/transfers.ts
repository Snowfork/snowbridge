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
            const key = `${chain.kind}_${chain.id}` as const
            const parachain = registry.kusama.parachains[key]
            if (!parachain) throw Error(`Cannot find chain ${key}`)
            return {
                id: parachain.id,
                kind: parachain.kind,
                key: parachain.key,
                parachain,
            }
        }
        case "polkadot": {
            const key = `${chain.kind}_${chain.id}` as const
            const parachain = registry.parachains[key]
            if (!parachain) throw Error(`Cannot find chain ${key}`)
            return {
                id: parachain.id,
                kind: parachain.kind,
                key: parachain.key,
                parachain,
            }
        }
        case "ethereum": {
            const key = `${chain.kind}_${chain.id}` as const
            const ethChain = registry.ethereumChains[key]
            if (!ethChain) throw Error(`Cannot find chain ${key}`)
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
            const key = `${chain.kind}_${chain.id}` as const
            const ethChain = registry.ethereumChains[key]
            if (!ethChain) throw Error(`Cannot find chain ${key}`)
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
