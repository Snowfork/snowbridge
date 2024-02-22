import { Context } from "./index"
import { assetStatusInfo, bridgeStatusInfo, channelStatusInfo } from "./status"
import { paraIdToChannelId } from "./utils"

export type SendValidationSuccess = {
    ethereumChainId: bigint,
}
export type SendValidationFailure = {
    bridgeOperational: boolean
    channelOperational: boolean
    lightClientLatencyIsAcceptable: boolean
    lightClientLatencySeconds: number
    tokenIsValidERC20: boolean,
    tokenIsRegistered: boolean,
    foreignAssetExists: boolean,
}
export type SendValidationResult = SendValidationSuccess | SendValidationFailure

export const validateSend = async (context: Context, sourceAccount: string, tokenAddress: string): Promise<SendValidationResult> => {

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
    ])

    const assetHub = assetHubParaId.toPrimitive() as number;
    const assetHubChannelId = paraIdToChannelId(assetHub)

    const [channelStatus, bridgeStatus] = await Promise.all([
        channelStatusInfo(context, assetHubChannelId),
        bridgeStatusInfo(context),
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])
    const bridgeOperational = bridgeStatus.toPolkadot.operatingMode.outbound === 'Normal' && bridgeStatus.toPolkadot.operatingMode.beacon === 'Normal'
    const channelOperational = channelStatus.toPolkadot.operatingMode.outbound === 'Normal'
    const lightClientLatencyIsAcceptable = bridgeStatus.toEthereum.latencySeconds < (60 * 60 * 3) // 3 Hours
    
    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress)
    const tokenIsRegistered = assetInfo.isTokenRegistered
    const tokenIsValidERC20 = assetInfo.isTokenRegistered
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    // TODO: user has asset
    let balance = await context.polkadot.api.assetHub.query.foreignAssets.account()
    // TODO: user has fees

    // Success
    // TODO: Display fees

    const canSend = bridgeOperational && channelOperational && lightClientLatencyIsAcceptable 
        && tokenIsRegistered && foreignAssetExists && tokenIsValidERC20

    if (canSend) {
        return {
            ethereumChainId: assetInfo.ethereumChainId,
        }
    } else {
        return {
            bridgeOperational,
            channelOperational,
            lightClientLatencyIsAcceptable,
            lightClientLatencySeconds: bridgeStatus.toEthereum.latencySeconds,
            tokenIsValidERC20,
            tokenIsRegistered,
            foreignAssetExists,
        }
    }
}
