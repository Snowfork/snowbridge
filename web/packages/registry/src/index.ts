export * from "./transfers"

import polkadot_mainnet from "./polkadot_mainnet_bridge_info.g"
import westend_sepolia from "./westend_sepolia_bridge_info.g"
import paseo_sepolia from "./paseo_sepolia_bridge_info.g"

export { paseo_sepolia, westend_sepolia, polkadot_mainnet }

import { BridgeInfo } from "@snowbridge/base-types"

export function bridgeInfoFor(
    env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {}),
): Readonly<BridgeInfo> {
    switch (env) {
        case "polkadot_mainnet":
            return polkadot_mainnet satisfies BridgeInfo
            break
        case "westend_sepolia":
            return westend_sepolia satisfies BridgeInfo
        case "paseo_sepolia":
            return paseo_sepolia satisfies BridgeInfo
        default:
            throw Error(`Unknown env '${env}'`)
    }
}
