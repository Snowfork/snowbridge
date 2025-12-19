import { AbstractProvider } from "ethers"
import { ApiPromise } from "@polkadot/api"
import { IERC20__factory } from "@snowbridge/contract-types"
import { ParachainBase } from "./parachains/parachainBase"
import { Asset } from "@snowbridge/base-types"

export const ETHER_TOKEN_ADDRESS = "0x0000000000000000000000000000000000000000"

export async function getAssetHubConversionPalletSwap(
    assetHub: ApiPromise,
    asset1: any,
    asset2: any,
    exactAsset2Balance: bigint,
) {
    const result = await assetHub.call.assetConversionApi.quotePriceTokensForExactTokens(
        asset1,
        asset2,
        exactAsset2Balance,
        true,
    )
    const asset1Balance = result.toPrimitive() as any
    if (asset1Balance == null) {
        throw Error(
            `No pool set up in asset conversion pallet for '${JSON.stringify(
                asset1,
            )}' and '${JSON.stringify(asset2)}'.`,
        )
    }
    return BigInt(asset1Balance)
}

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

export async function swapAsset1ForAsset2(
    assetHub: ApiPromise,
    asset1: any,
    asset2: any,
    exactAsset1Balance: bigint,
) {
    const result = await assetHub.call.assetConversionApi.quotePriceExactTokensForTokens(
        asset1,
        asset2,
        exactAsset1Balance,
        true,
    )
    const asset2Balance = result.toPrimitive() as any
    if (asset2Balance == null) {
        throw Error(
            `No pool set up in asset conversion pallet for '${JSON.stringify(
                asset1,
            )}' and '${JSON.stringify(asset2)}'.`,
        )
    }
    return BigInt(asset2Balance)
}

export async function validateAccount(
    parachainImpl: ParachainBase,
    beneficiaryAddress: string,
    ethChainId: number,
    tokenAddress: string,
    assetMetadata?: Asset,
    maxConsumers?: bigint,
) {
    // Check if the account is created
    const [beneficiaryAccount, beneficiaryTokenBalance] = await Promise.all([
        parachainImpl.getNativeAccount(beneficiaryAddress),
        parachainImpl.getTokenBalance(beneficiaryAddress, ethChainId, tokenAddress, assetMetadata),
    ])
    return {
        accountExists: !(
            beneficiaryAccount.consumers === 0n &&
            beneficiaryAccount.providers === 0n &&
            beneficiaryAccount.sufficients === 0n
        ),
        accountMaxConsumers:
            beneficiaryAccount.consumers >= (maxConsumers ?? 63n) && beneficiaryTokenBalance === 0n,
    }
}
