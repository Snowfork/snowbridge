import { xxhashAsHex } from "@polkadot/util-crypto"
import { Context } from "./index"
import { assetStatusInfo, bridgeStatusInfo } from "./status"
import { u8aToHex } from "@polkadot/util"
import { IKeyringPair } from "@polkadot/types/types"

export type SendValidationResult = {
    success?: {
        ethereumChainId: bigint
        assetHub: {
            validatedAt: string
            paraId: number
        }
        bridgeHub: {
            validatedAt: string
            paraId: number
        }
        sourceAddress: string
        beneficiary: string
        feeInDOT: bigint
        amount: bigint
        multiLocation: object
    }
    failure?: {
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
}

export const validateSend = async (context: Context, source: IKeyringPair, beneficiary: string, tokenAddress: string, amount: bigint, options = { defaultFee: 2_750_872_500_000n, acceptableLatencyInSeconds: 10800 /* 3 Hours */ }): Promise<SendValidationResult> => {

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
        let account = (await context.polkadot.api.assetHub.query.foreignAssets.account(assetInfo.multiLocation, source.address)).toPrimitive() as any
        if (account !== null) {
            assetBalance = BigInt(account.balance)
        }
    }
    const hasAsset = assetBalance >= amount
    // 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const [feeStorageItem, account] = await Promise.all([
        context.polkadot.api.assetHub.rpc.state.getStorage(feeStorageKey),
        context.polkadot.api.assetHub.query.system.account(source.address),
    ])
    const fee = BigInt((feeStorageItem as any).toPrimitive() || options.defaultFee)
    const dotBalance = BigInt((account.toPrimitive() as any).data.free)
    const canPayFee = fee < dotBalance
    console.log(fee, (account.toPrimitive() as any).data.free)

    const canSend = bridgeOperational && lightClientLatencyIsAcceptable
        && tokenIsRegistered && foreignAssetExists && tokenIsValidERC20 && hasAsset && canPayFee

    if (canSend) {
        return {
            success: {
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
                sourceAddress: source.address,
                beneficiary,
                amount,
                multiLocation: assetInfo.multiLocation
            }
        }
    } else {
        return {
            failure: {
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
}

export type SendResult = {
    success?: boolean,
    failure?: boolean,
}

export const send = async (context: Context, signer: IKeyringPair, plan: SendValidationResult, options = {
    xcm_version: 3
}): Promise<SendResult> => {
    if (plan.success) {
        console.log(plan.success)
        const assets: { [key: string]: any } = {}
        assets[`v${options.xcm_version}`] = [{
            id: { Concrete: plan.success.multiLocation },
            fun: { Fungible: plan.success.amount }
        }]
        const destination: { [key: string]: any } = {}
        destination[`v${options.xcm_version}`] = {
            parents: 2,
            interior: { X1: [ { GlobalConsensus: { Ethereum: { chain_id: plan.success.ethereumChainId } } } ] }
        }
        const beneficiary: { [key: string]: any } = {}
        beneficiary[`v${options.xcm_version}`] = {
            parents: 0,
            interior: { X1: [ { AccountKey20: { key: plan.success.beneficiary } } ] }
        }
        const fee_asset = 0
        const weight = "Unlimited"
        let result = await context.polkadot.api.assetHub.tx.polkadotXcm.transferAssets(
            destination,
            beneficiary,
            assets,
            fee_asset,
            weight
        ).signAndSend(signer)

        console.log(result.toPrimitive())
        return {
            success: true
        };
    }
    else {
        throw Error("plan failed")
    }
}