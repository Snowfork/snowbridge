import { AssetRegistry, Parachain } from "@snowbridge/base-types"

export const ETHER_TOKEN_ADDRESS = "0x0000000000000000000000000000000000000000"
export const DOT_LOCATION = { parents: 1, interior: "Here" }

export function findL2TokenAddress(
    registry: AssetRegistry,
    l2ChainId: number,
    tokenAddress: string,
): string | undefined {
    const l2Chain = registry.ethereumChains[`ethereum_l2_${l2ChainId}`]
    if (!l2Chain) {
        return undefined
    }
    for (const [l2TokenAddress, asset] of Object.entries(l2Chain.assets)) {
        if (asset.swapTokenAddress?.toLowerCase() === tokenAddress.toLowerCase()) {
            return l2TokenAddress
        }
    }
    return undefined
}

// Returns the bridged-ether `min_balance` registered in the AH foreign-assets
// pallet. Callers add this to `executionFee` for non-ETH transfers so the
// post-PayFees ether surplus deposited to a fresh recipient meets
// `Token::BelowMinimum` — otherwise the dust traps the entire DepositAsset.
export function getAssetHubEtherMinBalance(registry: AssetRegistry): bigint {
    const metadata =
        registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[ETHER_TOKEN_ADDRESS]
    if (!metadata) {
        throw Error("Bridged ether not registered on asset hub.")
    }
    return metadata.minimumBalance
}

export function supportsEthereumToPolkadotV2(parachain: Parachain): boolean {
    return (
        parachain.features.hasXcmPaymentApi &&
        parachain.features.xcmVersion === "v5" &&
        parachain.features.supportsV2
    )
}

export function supportsPolkadotToEthereumV2(parachain: Parachain): boolean {
    return (
        parachain.features.hasEthBalance &&
        parachain.features.hasXcmPaymentApi &&
        parachain.features.supportsAliasOrigin &&
        parachain.features.xcmVersion === "v5" &&
        parachain.features.supportsV2
    )
}
