import { AbstractProvider } from "ethers"
import { IERC20__factory } from "@snowbridge/contract-types"
import { AssetRegistry, Parachain } from "@snowbridge/base-types"

export const ETHER_TOKEN_ADDRESS = "0x0000000000000000000000000000000000000000"

export async function erc20Balance(
    ethereum: AbstractProvider,
    tokenAddress: string,
    owner: string,
    spender: string,
) {
    const tokenContract = IERC20__factory.connect(tokenAddress, ethereum)
    const [balance, gatewayAllowance] = await Promise.all([
        tokenContract.balanceOf(owner),
        tokenContract.allowance(owner, spender),
    ])
    return {
        balance,
        gatewayAllowance,
    }
}
export function findL2TokenAddress(
    registry: AssetRegistry,
    l2ChainId: number,
    tokenAddress: string,
): string | undefined {
    const l2Chain = registry.ethereumChains[l2ChainId]
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

export function supportsEthereumToPolkadotV2(parachain: Parachain): boolean {
  return (
    parachain.features.hasXcmPaymentApi &&
    parachain.features.xcmVersion === "v5" &&
    parachain.features.supportsV2
  );
}

export function supportsPolkadotToEthereumV2(parachain: Parachain): boolean {
  return (
    parachain.features.hasEthBalance &&
    parachain.features.hasXcmPaymentApi &&
    parachain.features.supportsAliasOrigin &&
    parachain.features.xcmVersion === "v5" &&
    parachain.features.supportsV2
  );
}
