import {
    ChainMap,
    ChainRef,
    ChainKey,
    ChainKind,
    EthereumChain,
    Parachain,
} from "@snowbridge/base-types"

type RegistryChain = EthereumChain | Parachain

const FRIENDLY_NAME_OVERRIDES: Partial<Record<ChainKey<ChainKind>, string>> = {
    ethereum_1: "ethereum",
    polkadot_1000: "assetHub",
    kusama_1000: "kusamaAssetHub",
    polkadot_2004: "moonbeamSubstrate",
}

function wordsForFriendlyName(value: string): string[] {
    return value
        .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
        .replace(/[^a-zA-Z0-9]+/g, " ")
        .trim()
        .split(/\s+/)
        .filter(Boolean)
}

function camelCase(value: string): string {
    const words = wordsForFriendlyName(value)
    if (words.length === 0) return ""
    return words
        .map((word, index) => {
            const lower = word.toLowerCase()
            return index === 0 ? lower : lower[0].toUpperCase() + lower.slice(1)
        })
        .join("")
}

function chainDisplayName(chain: RegistryChain, allChains: RegistryChain[]): string | undefined {
    if ("info" in chain) return chain.info.name
    if (chain.name?.trim()) return chain.name
    if (chain.evmParachainId !== undefined) {
        const linkedParachain = allChains.find(
            (candidate) => candidate.kind === "polkadot" && candidate.id === chain.evmParachainId,
        )
        if (linkedParachain && "info" in linkedParachain) {
            return linkedParachain.info.name
        }
    }
    return undefined
}

export function buildFriendlyChains(chains: RegistryChain[]): ChainMap {
    const chainsByName: ChainMap = {}
    for (const chain of chains) {
        const displayName = chainDisplayName(chain, chains)
        const name =
            FRIENDLY_NAME_OVERRIDES[chain.key] ??
            (displayName ? camelCase(displayName) : undefined) ??
            chain.key
        if (name in chainsByName) {
            const existing = chainsByName[name]
            throw Error(
                `Duplicate friendly chain name '${name}' for ${existing.key} and ${chain.key}.`,
            )
        }
        chainsByName[name] = {
            key: chain.key,
            kind: chain.kind,
            id: chain.id,
        } satisfies ChainRef
    }
    return chainsByName
}
