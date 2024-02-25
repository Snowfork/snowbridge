import { xxhashAsHex } from "@polkadot/util-crypto"
import { Context } from "./index"
import { assetStatusInfo, bridgeStatusInfo } from "./status"
import { u8aToHex } from "@polkadot/util"

export type SendValidationSuccess = {
    ethereumChainId: bigint
    assetHub: {
        validatedAt: string,
        paraId: number,
    },
    bridgeHub: {
        validatedAt: string,
        paraId: number,
    },
    beneficiary: string
    feeInDOT: bigint
}
export type SendValidationFailure = {
    bridgeOperational: boolean
    lightClientLatencyIsAcceptable: boolean
    lightClientLatencySeconds: number
    lightClientLatencyBlocks: number
    tokenIsValidERC20: boolean
    tokenIsRegistered: boolean
    foreignAssetExists: boolean
    hasAsset: boolean
    assetBalance: bigint
    canPayFee: boolean
    dotBalance: bigint
}
export type SendValidationResult = SendValidationSuccess | SendValidationFailure

export const validateSend = async (context: Context, source: string, beneficiary: string, tokenAddress: string, amount: bigint, options = { defaultFee: 2_750_872_500_000n, acceptableLatencyInSeconds: 10800 /* 3 Hours */ }): Promise<SendValidationResult> => {

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
    ])

    const [bridgeStatus] = await Promise.all([
        bridgeStatusInfo(context),
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])
    const bridgeOperational = bridgeStatus.toEthereum.operatingMode.outbound === 'Normal'
    const lightClientLatencyIsAcceptable = bridgeStatus.toEthereum.latencySeconds < options.acceptableLatencyInSeconds

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress)
    const tokenIsRegistered = assetInfo.isTokenRegistered
    const tokenIsValidERC20 = assetInfo.isTokenRegistered
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    let assetBalance = 0n
    if (foreignAssetExists) {
        let account = (await context.polkadot.api.assetHub.query.foreignAssets.account(assetInfo.multiLocation, source)).toPrimitive() as any
        if (account !== null) {
            assetBalance = BigInt(account.balance)
        }
    }
    const hasAsset = assetBalance >= amount
    // 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const [feeStorageItem, account] = await Promise.all([
        context.polkadot.api.assetHub.rpc.state.getStorage(feeStorageKey),
        context.polkadot.api.assetHub.query.system.account(source),
    ])
    const fee = BigInt((feeStorageItem as any).toPrimitive() || options.defaultFee)
    const dotBalance = BigInt((account.toPrimitive() as any).data.free)
    const canPayFee = fee < dotBalance
    console.log(fee, (account.toPrimitive() as any).data.free)

    const canSend = bridgeOperational && lightClientLatencyIsAcceptable
        && tokenIsRegistered && foreignAssetExists && tokenIsValidERC20 && hasAsset && canPayFee

    if (canSend) {
        return {
            ethereumChainId: assetInfo.ethereumChainId,
            assetHub: {
                paraId: assetHubParaId.toPrimitive() as number,
                validatedAt: u8aToHex(assetHubHead),
            },
            bridgeHub: {
                paraId: bridgeHubParaId.toPrimitive() as number,
                validatedAt: u8aToHex(bridgeHubHead),
            },
            feeInDOT: fee,
            beneficiary
        }
    } else {
        return {
            bridgeOperational,
            lightClientLatencyIsAcceptable,
            lightClientLatencySeconds: bridgeStatus.toEthereum.latencySeconds,
            lightClientLatencyBlocks: bridgeStatus.toEthereum.blockLatency,
            tokenIsValidERC20,
            tokenIsRegistered,
            foreignAssetExists,
            hasAsset,
            assetBalance,
            canPayFee,
            dotBalance,
        }
    }
}
