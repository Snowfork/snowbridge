import { ApiPromise } from "@polkadot/api"
import { Codec, Registry } from "@polkadot/types/types"
import { IERC20Metadata__factory, IERC20__factory } from "@snowbridge/contract-types"
import { Context } from "./index"

export interface NativeAsset {
    tokenSymbol: string
    tokenDecimal: number
    ss58Format: number
}

export const parachainNativeAsset = async (api: ApiPromise): Promise<NativeAsset> => {
    const properties = await api.rpc.system.properties()
    const tokenSymbols = properties.tokenSymbol.unwrapOrDefault()
    const tokenDecimals = properties.tokenDecimals.unwrapOrDefault()

    return {
        tokenSymbol: tokenSymbols.at(0)?.toString() ?? "DOT",
        tokenDecimal: tokenDecimals.at(0)?.toNumber() ?? 10,
        ss58Format: properties.ss58Format.unwrapOr(null)?.toNumber() ?? 42,
    }
}

export const erc20TokenToAssetLocation = (
    registry: Registry,
    ethChainId: bigint,
    token: string
) => {
    return registry.createType("StagingXcmV3MultiLocation", {
        parents: 2,
        interior: {
            X2: [
                { GlobalConsensus: { Ethereum: { chain_id: ethChainId } } },
                { AccountKey20: { key: token } },
            ],
        },
    })
}

export const assetStatusInfo = async (
    context: Context,
    tokenAddress: string,
    ownerAddress?: string
) => {
    let [ethereumNetwork, isTokenRegistered] = await Promise.all([
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress),
    ])

    const ethereumChainId = ethereumNetwork.chainId
    const multiLocation = erc20TokenToAssetLocation(
        context.polkadot.api.assetHub.registry,
        ethereumChainId,
        tokenAddress
    )
    const foreignAsset = (
        await context.polkadot.api.assetHub.query.foreignAssets.asset(multiLocation)
    ).toPrimitive() as { status: "Live" }

    const tokenContract = IERC20__factory.connect(tokenAddress, context.ethereum.api)
    let ownerBalance = BigInt(0)
    let tokenGatewayAllowance = BigInt(0)
    let isValidERC20 = true
    try {
        const erc20balance = await assetErc20Balance(
            context,
            tokenAddress,
            ownerAddress ?? "0x0000000000000000000000000000000000000000"
        )
        ownerBalance = erc20balance.balance
        tokenGatewayAllowance = erc20balance.gatewayAllowance
    } catch {
        isValidERC20 = false
    }

    return {
        ethereumChainId,
        multiLocation,
        isValidERC20,
        tokenContract,
        isTokenRegistered,
        tokenGatewayAllowance,
        ownerBalance,
        foreignAssetExists: foreignAsset !== null,
        foreignAsset,
    }
}

export const assetErc20Balance = async (
    context: Context,
    token: string,
    owner: string
): Promise<{
    balance: bigint
    gatewayAllowance: bigint
}> => {
    const tokenContract = IERC20__factory.connect(token, context.ethereum.api)
    const gateway = await context.ethereum.contracts.gateway.getAddress()
    const [balance, gatewayAllowance] = await Promise.all([
        tokenContract.balanceOf(owner),
        tokenContract.allowance(owner, gateway),
    ])
    return {
        balance,
        gatewayAllowance,
    }
}

export type ERC20Metadata = {
    name: string
    symbol: string
    decimals: bigint
}

export const assetErc20Metadata = async (
    context: Context,
    tokenAddress: string
): Promise<ERC20Metadata> => {
    const tokenMetadata = IERC20Metadata__factory.connect(tokenAddress, context.ethereum.api)
    const [name, symbol, decimals] = await Promise.all([
        tokenMetadata.name(),
        tokenMetadata.symbol(),
        tokenMetadata.decimals(),
    ])
    return { name, symbol, decimals }
}

export const palletAssetsBalance = async (
    api: ApiPromise,
    location: Codec,
    address: string,
    instance = "assets"
): Promise<bigint | null> => {
    let account = (await api.query[instance].account(location, address)).toPrimitive() as any
    if (account !== null) {
        return BigInt(account.balance)
    }
    return null
}
