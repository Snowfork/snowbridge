import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import { AssetRegistry } from "@snowbridge/base-types"
import { CallDryRunEffects, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import {
    DeliveryFee,
    dryRunBridgeHub,
    resolveInputs,
    Transfer,
    ValidationKind,
    ValidationLog,
    ValidationReason,
    ValidationResult,
} from "./toEthereum_v2"
import { PNAFromAH } from "./transfers/toEthereum/pnaFromAH"
import { TransferInterface } from "./transfers/toEthereum/transferInterface"
import { ERC20FromAH } from "./transfers/toEthereum/erc20FromAH"
import { PNAFromParachain } from "./transfers/toEthereum/pnaFromParachain"
import { ERC20FromParachain } from "./transfers/toEthereum/erc20FromParachain"
import {
    isRelaychainLocation,
    isParachainNative,
    DOT_LOCATION,
    HERE_LOCATION,
    bridgeLocation,
} from "./xcmBuilder"
import { xxhashAsHex } from "@polkadot/util-crypto"
import { BN } from "@polkadot/util"
import { padFeeByPercentage } from "./utils"
import { paraImplementation } from "./parachains"
import { Context } from "./index"
import { ETHER_TOKEN_ADDRESS, getAssetHubConversionPalletSwap } from "./assets_v2"
import { getOperatingStatus } from "./status"
import { CreateAgent } from "./registration/agent/createAgent"
import { Wallet, TransactionReceipt } from "ethers"

export { ValidationKind, signAndSend } from "./toEthereum_v2"

export function createTransferImplementation(
    sourceParaId: number,
    registry: AssetRegistry,
    tokenAddress: string,
): TransferInterface {
    const { sourceAssetMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)

    let transferImpl
    if (sourceParaId == registry.assetHubParaId) {
        if (sourceAssetMetadata.location) {
            transferImpl = new PNAFromAH()
        } else {
            transferImpl = new ERC20FromAH()
        }
    } else {
        if (sourceAssetMetadata.location) {
            transferImpl = new PNAFromParachain()
        } else {
            transferImpl = new ERC20FromParachain()
        }
    }
    return transferImpl
}

export async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string,
) {
    const origin = { system: { signed: sourceAccount } }
    // To ensure compatibility, dryRunCall includes the version parameter in XCMv5.
    let result
    try {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx.inner.toHex(), 5)
    } catch {
        result = await source.call.dryRunApi.dryRunCall<
            Result<CallDryRunEffects, XcmDryRunApiError>
        >(origin, tx.inner.toHex())
    }

    let assetHubForwarded
    let bridgeHubForwarded
    const success = result.isOk && result.asOk.executionResult.isOk
    if (!success) {
        console.error(
            "Error during dry run on source parachain:",
            sourceAccount,
            tx.toHuman(),
            result.toHuman(),
        )
    } else {
        bridgeHubForwarded =
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            }) ??
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
                )
            })
        assetHubForwarded =
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === assetHubParaId
                )
            }) ??
            result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV5 &&
                    x[0].asV5.parents.toNumber() === 1 &&
                    x[0].asV5.interior.isX1 &&
                    x[0].asV5.interior.asX1[0].isParachain &&
                    x[0].asV5.interior.asX1[0].asParachain.toNumber() === assetHubParaId
                )
            })
    }
    return {
        success: success && (bridgeHubForwarded || assetHubForwarded),
        error:
            result.isOk && result.asOk.executionResult.isErr
                ? result.asOk.executionResult.asErr.toJSON()
                : undefined,
        assetHubForwarded,
        bridgeHubForwarded,
    }
}

export async function dryRunAssetHub(
    assetHub: ApiPromise,
    parachainId: number,
    bridgeHubParaId: number,
    xcm: any,
) {
    const sourceParachain = { v5: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded
    let bridgeHubForwarded
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV5 &&
                x[0].asV5.parents.toNumber() === 1 &&
                x[0].asV5.interior.isX1 &&
                x[0].asV5.interior.asX1[0].isParachain &&
                x[0].asV5.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
            )
        })
        sourceParachainForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV5 &&
                x[0].asV5.parents.toNumber() === 1 &&
                x[0].asV5.interior.isX1 &&
                x[0].asV5.interior.asX1[0].isParachain &&
                x[0].asV5.interior.asX1[0].asParachain.toNumber() === parachainId
            )
        })
    }
    return {
        success: success && bridgeHubForwarded,
        sourceParachainForwarded,
        bridgeHubForwarded,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}

export const MaxWeight = { refTime: 15_000_000_000n, proofSize: 800_000 }

export const isFeeAllowed = (feeLocation: any, sourceParaId: number) => {
    return isRelaychainLocation(feeLocation) || isParachainNative(feeLocation, sourceParaId)
}

export const getSnowbridgeDeliveryFee = async (assetHub: ApiPromise, defaultFee?: bigint) => {
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFeeV2:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")
    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        snowbridgeDeliveryFeeDOT = defaultFee ?? 150_000_000_000n
    } else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }
    return snowbridgeDeliveryFeeDOT
}

export type DeliveryXcm = {
    localXcm: any
    forwardedXcmToBH: any
    forwardXcmToAH?: any
    returnToSenderXcm?: any
}

export const estimateEthereumExecutionFee = async (
    context: Context,
    registry: AssetRegistry,
    sourceParaId: number,
    tokenAddress: string,
): Promise<bigint> => {
    const ethereum = await context.ethereum()
    const { tokenErcMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)

    // Calculate execution cost on ethereum
    let ethereumChain = registry.ethereumChains[registry.ethChainId.toString()]
    let feeData = await ethereum.getFeeData()
    let ethereumExecutionFee =
        (feeData.gasPrice ?? 2_000_000_000n) *
        ((tokenErcMetadata.deliveryGas ?? 80_000n) + (ethereumChain.baseDeliveryGas ?? 120_000n))
    return ethereumExecutionFee
}

export const estimateFeesFromAssetHub = async (
    context: Context,
    registry: AssetRegistry,
    tokenAddress: string,
    deliveryXcm: DeliveryXcm,
    options?: {
        padPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
    },
): Promise<DeliveryFee> => {
    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await paraImplementation(assetHub)

    const feePadPercentage = options?.padPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = 0n
    let snowbridgeDeliveryFeeDOT = 0n

    localExecutionFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateXcmFee(deliveryXcm.localXcm, DOT_LOCATION),
        feePadPercentage,
    )

    bridgeHubDeliveryFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateDeliveryFeeInDOT(
            registry.bridgeHubParaId,
            deliveryXcm.forwardedXcmToBH,
        ),
        feePadPercentage,
    )

    snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(assetHub, options?.defaultFee)

    let totalFeeInDot =
        localExecutionFeeDOT +
        snowbridgeDeliveryFeeDOT +
        assetHubExecutionFeeDOT +
        returnToSenderExecutionFeeDOT +
        returnToSenderDeliveryFeeDOT +
        bridgeHubDeliveryFeeDOT

    let ethereumExecutionFee = await estimateEthereumExecutionFee(
        context,
        registry,
        registry.assetHubParaId,
        tokenAddress,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let returnToSenderExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let localExecutionFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                assetHub,
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        } else {
            throw new Error("Unsupported fee token location")
        }
    }

    return {
        localExecutionFeeDOT,
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        totalFeeInNative,
    }
}

export const estimateFeesFromParachains = async (
    context: Context,
    sourceParaId: number,
    registry: AssetRegistry,
    tokenAddress: string,
    deliveryXcm: DeliveryXcm,
    options?: {
        padPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
    },
): Promise<DeliveryFee> => {
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    const sourceParachainImpl = await paraImplementation(await context.parachain(sourceParaId))

    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await paraImplementation(assetHub)

    const feePadPercentage = options?.padPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let localDeliveryFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = 0n
    let snowbridgeDeliveryFeeDOT = 0n

    let localExecutionFeeInNative: bigint | undefined = undefined
    let localDeliveryFeeInNative: bigint | undefined = undefined
    let returnToSenderExecutionFeeNative: bigint | undefined = undefined
    if (sourceParachain.features.hasDotBalance) {
        localExecutionFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.localXcm, DOT_LOCATION),
            feePadPercentage,
        )
        localDeliveryFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateDeliveryFeeInDOT(
                registry.assetHubParaId,
                deliveryXcm.forwardXcmToAH,
            ),
            feePadPercentage,
        )
        returnToSenderExecutionFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.returnToSenderXcm, DOT_LOCATION),
            feePadPercentage,
        )
    } else {
        localExecutionFeeInNative = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.localXcm, HERE_LOCATION),
            feePadPercentage,
        )
        localDeliveryFeeInNative = padFeeByPercentage(
            await sourceParachainImpl.calculateDeliveryFeeInNative(
                registry.assetHubParaId,
                deliveryXcm.forwardXcmToAH,
            ),
            feePadPercentage,
        )
        returnToSenderExecutionFeeNative = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(deliveryXcm.returnToSenderXcm, HERE_LOCATION),
            feePadPercentage,
        )
    }

    returnToSenderDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
        sourceParaId,
        deliveryXcm.returnToSenderXcm,
    )
    assetHubExecutionFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateXcmFee(deliveryXcm.forwardXcmToAH, DOT_LOCATION),
        feePadPercentage,
    )

    bridgeHubDeliveryFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateDeliveryFeeInDOT(
            registry.bridgeHubParaId,
            deliveryXcm.forwardedXcmToBH,
        ),
        feePadPercentage,
    )

    snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(assetHub, options?.defaultFee)

    let totalFeeInDot =
        localExecutionFeeDOT +
        localDeliveryFeeDOT +
        snowbridgeDeliveryFeeDOT +
        assetHubExecutionFeeDOT +
        returnToSenderExecutionFeeDOT +
        returnToSenderDeliveryFeeDOT +
        bridgeHubDeliveryFeeDOT

    let ethereumExecutionFee = await estimateEthereumExecutionFee(
        context,
        registry,
        sourceParaId,
        tokenAddress,
    )

    // calculate the cost of swapping in native asset
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let ethereumExecutionFeeInNative: bigint | undefined
    let feeLocation = options?.feeTokenLocation
    if (feeLocation) {
        // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
        if (isRelaychainLocation(feeLocation)) {
            ethereumExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                assetHub,
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInNative
            totalFeeInNative = totalFeeInDot
        }
        // On Parachains, we can use their native asset as the fee token.
        // If the fee is in native, we need to swap it to DOT first, then swap DOT to Ether to cover the ethereum execution fee.
        else if (isParachainNative(feeLocation, sourceParaId)) {
            let ethereumExecutionFeeInDOT = await getAssetHubConversionPalletSwap(
                assetHub,
                DOT_LOCATION,
                bridgeLocation(registry.ethChainId),
                padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage),
            )
            ethereumExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                assetHub,
                feeLocation,
                DOT_LOCATION,
                padFeeByPercentage(ethereumExecutionFeeInDOT, feeSlippagePadPercentage),
            )
            totalFeeInDot += ethereumExecutionFeeInDOT
            totalFeeInNative = await getAssetHubConversionPalletSwap(
                assetHub,
                feeLocation,
                DOT_LOCATION,
                padFeeByPercentage(totalFeeInDot, feeSlippagePadPercentage),
            )
            if (localExecutionFeeInNative) {
                totalFeeInNative += localExecutionFeeInNative
            }
            if (localDeliveryFeeInNative) {
                totalFeeInNative += localDeliveryFeeInNative
            }
            if (returnToSenderExecutionFeeNative) {
                totalFeeInNative += returnToSenderExecutionFeeNative
            }
        } else {
            throw new Error("Unsupported fee token location")
        }
    }

    return {
        localExecutionFeeDOT,
        localDeliveryFeeDOT,
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        localDeliveryFeeInNative,
        totalFeeInNative,
    }
}

export const validateTransferFromAssetHub = async (
    context: Context,
    transfer: Transfer,
): Promise<ValidationResult> => {
    const { registry, fee, tokenAddress, amount } = transfer.input
    const { sourceAccountHex, sourceParaId, sourceAssetMetadata } = transfer.computed
    const { tx } = transfer

    const { sourceParachain, gateway, bridgeHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  bridgeHub: await context.bridgeHub(),
              }
            : context

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
    let dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    let tokenBalance: any
    let isNativeBalance = false
    // For DOT on AH, get it from the native balance pallet.
    if (
        transfer.computed.sourceAssetMetadata.location &&
        isRelaychainLocation(transfer.computed.sourceAssetMetadata.location)
    ) {
        tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
        isNativeBalance = true
    } else {
        tokenBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata,
        )
    }
    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    } else {
        if (amount > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }

    // No fee specified means that the fee.ethereumExecutionFee is paid in Ether on source chain.
    if (!fee.feeLocation) {
        let etherBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            ETHER_TOKEN_ADDRESS,
        )

        if (fee.ethereumExecutionFee! > etherBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientEtherBalance,
                message: "Insufficient ether balance to submit transaction.",
            })
        }
    }

    let sourceDryRunError
    let assetHubDryRunError
    let bridgeHubDryRunError
    // do the dry run, get the forwarded xcm and dry run that
    const dryRunResultAssetHub = await dryRunOnSourceParachain(
        sourceParachain,
        registry.assetHubParaId,
        registry.bridgeHubParaId,
        transfer.tx,
        sourceAccountHex,
    )
    if (dryRunResultAssetHub.success && dryRunResultAssetHub.bridgeHubForwarded) {
        const dryRunResultBridgeHub = await dryRunBridgeHub(
            bridgeHub,
            registry.assetHubParaId,
            dryRunResultAssetHub.bridgeHubForwarded[1][0],
        )
        if (!dryRunResultBridgeHub.success) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run failed on Bridge Hub.",
            })
            bridgeHubDryRunError = dryRunResultBridgeHub.errorMessage
        }
    } else {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.DryRunFailed,
            message: "Dry run call failed on Asset Hub.",
        })
        assetHubDryRunError = dryRunResultAssetHub.error
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    // recheck total after fee estimation
    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative + sourceExecutionFee > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }
    if (sourceExecutionFee + fee.totalFeeInDot > dotBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientDotFee,
            message: "Insufficient DOT balance to submit transaction on the source parachain.",
        })
    }
    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.BridgeStatusNotOperational,
            message: "Bridge operations have been paused by onchain governance.",
        })
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError,
            bridgeHubDryRunError,
        },
        transfer,
    }
}

export const validateTransferFromParachain = async (
    context: Context,
    transfer: Transfer,
): Promise<ValidationResult> => {
    const { registry, fee, tokenAddress, amount } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        sourceParachain: source,
        sourceAssetMetadata,
    } = transfer.computed
    const { tx } = transfer

    const { sourceParachain, gateway, bridgeHub, assetHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  bridgeHub: await context.bridgeHub(),
                  assetHub: await context.assetHub(),
              }
            : context

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
    let dotBalance: bigint | undefined = undefined
    if (source.features.hasDotBalance) {
        dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    }
    let tokenBalance: any
    let isNativeBalance = false

    isNativeBalance =
        sourceAssetMetadata.decimals === source.info.tokenDecimals &&
        sourceAssetMetadata.symbol == source.info.tokenSymbols
    if (isNativeBalance) {
        tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
    } else {
        tokenBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata,
        )
    }

    if (isNativeBalance && fee.totalFeeInNative) {
        if (amount + fee.totalFeeInNative > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    } else {
        if (amount > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }

    if (!fee.feeLocation) {
        let etherBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            ETHER_TOKEN_ADDRESS,
        )

        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        if (fee.ethereumExecutionFee! > etherBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientEtherBalance,
                message: "Insufficient ether balance to submit transaction.",
            })
        }
    }

    let sourceDryRunError
    let assetHubDryRunError
    let bridgeHubDryRunError
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(
            sourceParachain,
            registry.assetHubParaId,
            registry.bridgeHubParaId,
            transfer.tx,
            sourceAccountHex,
        )
        if (!dryRunSource.success) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run call on source failed.",
            })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success) {
            if (!dryRunSource.assetHubForwarded) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run call did not provide a forwared xcm.",
                })
            } else {
                const dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    dryRunSource.assetHubForwarded[1][0],
                )
                if (dryRunResultAssetHub.success && dryRunResultAssetHub.bridgeHubForwarded) {
                    const dryRunResultBridgeHub = await dryRunBridgeHub(
                        bridgeHub,
                        registry.assetHubParaId,
                        dryRunResultAssetHub.bridgeHubForwarded[1][0],
                    )
                    if (!dryRunResultBridgeHub.success) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message: "Dry run failed on Bridge Hub.",
                        })
                        bridgeHubDryRunError = dryRunResultBridgeHub.errorMessage
                    }
                } else {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run failed on Asset Hub.",
                    })
                    assetHubDryRunError = dryRunResultAssetHub.errorMessage
                }
            }
        }
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    const bridgeStatus = await getOperatingStatus({ gateway, bridgeHub })
    if (bridgeStatus.toEthereum.outbound !== "Normal") {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.BridgeStatusNotOperational,
            message: "Bridge operations have been paused by onchain governance.",
        })
    }

    const success = logs.find((l) => l.kind === ValidationKind.Error) === undefined

    return {
        logs,
        success,
        data: {
            bridgeStatus,
            nativeBalance,
            dotBalance,
            sourceExecutionFee,
            tokenBalance,
            sourceDryRunError,
            assetHubDryRunError,
            bridgeHubDryRunError,
        },
        transfer,
    }
}

// Agent creation exports
export type {
    AgentCreation,
    AgentCreationValidationResult,
    AgentCreationInterface,
} from "./registration/agent/agentInterface"

export function createAgentCreationImplementation() {
    return new CreateAgent()
}

export async function sendAgentCreation(
    creation: any,
    wallet: Wallet,
): Promise<TransactionReceipt> {
    const response = await wallet.sendTransaction(creation.tx)
    const receipt = await response.wait(1)
    if (!receipt) {
        throw Error(`Transaction ${response.hash} not included.`)
    }
    return receipt
}
