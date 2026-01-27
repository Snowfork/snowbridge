import {
    AssetRegistry,
    EthereumChain,
    Parachain,
    TransferRoute,
    SourceType,
    TransferLocation,
} from "@snowbridge/base-types"

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
