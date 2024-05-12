import { ApiPromise } from "@polkadot/api"
import { IERC20Metadata__factory, IERC20__factory } from "@snowbridge/contract-types"
import { Context } from './index'

export const getSubstrateToken = async (api: ApiPromise): Promise<{
    tokenSymbol: string
    tokenDecimal: number
    ss58Format: number | null
}> => {
    const properties = await api.rpc.system.properties()
    const tokenSymbols = properties.tokenSymbol.unwrapOrDefault()
    const tokenDecimals = properties.tokenDecimals.unwrapOrDefault()

    return {
        tokenSymbol: tokenSymbols.at(0)?.toString() ?? "DOT",
        tokenDecimal: tokenDecimals.at(0)?.toNumber() ?? 10,
        ss58Format: properties.ss58Format.unwrapOr(null)?.toNumber() ?? null,
    }
}

export const assetStatusInfo = async (context: Context, tokenAddress: string, ownerAddress?: string) => {
    let [ethereumNetwork, gatewayAddress, isTokenRegistered] = await Promise.all([
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.getAddress(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])

    const ethereumChainId = ethereumNetwork.chainId
    const multiLocation = context.polkadot.api.assetHub.createType('StagingXcmV3MultiLocation', {
        parents: 2,
        interior: {
            X2: [
                { GlobalConsensus: { Ethereum: { chain_id: ethereumChainId } } },
                { AccountKey20: { key: tokenAddress } },
            ]
        }
    })
    const foreignAsset = (await context.polkadot.api.assetHub.query.foreignAssets.asset(multiLocation)).toPrimitive() as { status: 'Live' }

    const tokenContract = IERC20__factory.connect(tokenAddress, context.ethereum.api)
    let ownerBalance = BigInt(0)
    let tokenGatewayAllowance = BigInt(0)
    let isValidERC20 = true
    try {
        const owner = ownerAddress || "0x0000000000000000000000000000000000000000"
        const [tokenBalance_, tokenGatewayAllowance_] = await Promise.all([
            tokenContract.balanceOf(owner),
            tokenContract.allowance(owner, gatewayAddress),
        ])
        ownerBalance = tokenBalance_
        tokenGatewayAllowance = tokenGatewayAllowance_
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

export const assetMetadata = async (context: Context, tokenAddress: string) => {
    const tokenMetadata = IERC20Metadata__factory.connect(tokenAddress, context.ethereum.api)
    const [name, symbol, decimal] = await Promise.all([
        tokenMetadata.name(),
        tokenMetadata.symbol(),
        tokenMetadata.decimals(),
    ])
    return { name, symbol, decimal }
}
