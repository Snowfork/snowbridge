import { AssetRegistry } from "@snowbridge/base-types"
import polkadot_mainnet from "./polkadot_mainnet_registry.g"
import westend_sepolia from "./westend_sepolia_registry.g"
import paseo_sepolia from "./paseo_sepolia_registry.g"

export function assetRegistryFor(
    env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {}),
): Readonly<AssetRegistry> {
    switch (env) {
        case "polkadot_mainnet":
            return polkadot_mainnet satisfies AssetRegistry
            break
        case "westend_sepolia":
            return westend_sepolia satisfies AssetRegistry
        case "paseo_sepolia":
            return paseo_sepolia satisfies AssetRegistry
        default:
            throw Error(`Unknown env '${env}'`)
    }
}
