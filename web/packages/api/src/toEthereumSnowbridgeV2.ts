import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    Parachain,
} from "@snowbridge/base-types"
import { CallDryRunEffects, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import {
    dryRunBridgeHub,
    resolveInputs,
    ValidationKind,
    ValidationLog,
    ValidationReason,
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
import { BN, hexToU8a } from "@polkadot/util"
import { padFeeByPercentage } from "./utils"
import { paraImplementation } from "./parachains"
import { Context } from "./index"
import {
    AggregatedAsset,
    ConcreteToken,
    ETHER_TOKEN_ADDRESS,
    getAssetHubConversionPalletSwap,
} from "./assets_v2"
import { getOperatingStatus, OperationStatus } from "./status"
import { AbstractProvider, ethers, Wallet, TransactionReceipt } from "ethers"
import { CreateAgent } from "./registration/agent/createAgent"

export { ValidationKind, signAndSend } from "./toEthereum_v2"

export type DeliveryFeeV2 = {
    snowbridgeDeliveryFeeDOT: bigint
    bridgeHubDeliveryFeeDOT: bigint
    assetHubExecutionFeeDOT: bigint
    totalFeeInDot: bigint
    localExecutionFeeDOT?: bigint
    localDeliveryFeeDOT?: bigint
    ethereumExecutionFee?: bigint
    feeLocation?: any
    totalFeeInNative?: bigint
    assetHubExecutionFeeNative?: bigint
    localExecutionFeeInNative?: bigint
    localDeliveryFeeInNative?: bigint
    ethereumExecutionFeeInNative?: bigint
}

export type TransferV2 = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokens: ConcreteToken[]
        fee: DeliveryFeeV2
        contractCall?: ContractCall
    }
    computed: {
        sourceParaId: number
        sourceParachain: Parachain
        sourceAccountHex: string
        aggregatedAssets: AggregatedAsset[]
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type ValidationResultV2 = {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance?: bigint
        sourceExecutionFee: bigint
        tokenBalances?: ConcreteToken[]
        sourceDryRunError: any
        assetHubDryRunError: any
        bridgeHubDryRunError?: any
    }
    transfer: TransferV2
}

export function createTransferImplementation(
    sourceParaId: number,
    registry: AssetRegistry,
    tokenAddresses: string[],
): TransferInterface {
    const { sourceAssetMetadata } = resolveInputs(registry, tokenAddresses[0], sourceParaId)

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
}

export const estimateEthereumExecutionFee = async (
    context: Context,
    registry: AssetRegistry,
    sourceParaId: number,
    tokenAddresses: string[],
    contractCall?: ContractCall,
): Promise<bigint> => {
    const ethereum = await context.ethereum()
    const ethereumChain = registry.ethereumChains[registry.ethChainId.toString()]
    const feeData = await ethereum.getFeeData()
    const gasPrice = feeData.gasPrice ?? 2_000_000_000n
    let ethereumExecutionFee: bigint = ethereumChain.baseDeliveryGas ?? 120_000n
    for (let tokenAddress of tokenAddresses) {
        const { tokenErcMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)
        ethereumExecutionFee += gasPrice * (tokenErcMetadata.deliveryGas ?? 80_000n)
    }
    ethereumExecutionFee += contractCall?.gas ?? 0n
    return ethereumExecutionFee
}

export const estimateFeesFromAssetHub = async (
    context: Context,
    registry: AssetRegistry,
    tokenAddresses: string[],
    deliveryXcm: DeliveryXcm,
    options?: {
        padPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
        contractCall?: ContractCall
    },
): Promise<DeliveryFeeV2> => {
    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await paraImplementation(assetHub)

    const feePadPercentage = options?.padPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
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
        bridgeHubDeliveryFeeDOT

    let ethereumExecutionFee = await estimateEthereumExecutionFee(
        context,
        registry,
        registry.assetHubParaId,
        tokenAddresses,
        options?.contractCall,
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
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        totalFeeInNative,
    }
}

export const estimateFeesFromParachains = async (
    context: Context,
    sourceParaId: number,
    registry: AssetRegistry,
    tokenAddresses: string[],
    deliveryXcm: DeliveryXcm,
    options?: {
        padPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
        feeTokenLocation?: any
        contractCall?: ContractCall
    },
): Promise<DeliveryFeeV2> => {
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    const sourceParachainImpl = await paraImplementation(await context.parachain(sourceParaId))

    const assetHub = await context.parachain(registry.assetHubParaId)
    const assetHubImpl = await paraImplementation(assetHub)

    const feePadPercentage = options?.padPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n

    let localExecutionFeeDOT = 0n
    let localDeliveryFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
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
    }

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
        bridgeHubDeliveryFeeDOT

    let ethereumExecutionFee = await estimateEthereumExecutionFee(
        context,
        registry,
        sourceParaId,
        tokenAddresses,
        options?.contractCall,
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
        totalFeeInDot,
        ethereumExecutionFee,
        feeLocation,
        assetHubExecutionFeeNative,
        ethereumExecutionFeeInNative,
        localExecutionFeeInNative,
        localDeliveryFeeInNative,
        totalFeeInNative,
    }
}

export const validateTransfer = async (
    context: Context,
    transfer: TransferV2,
): Promise<ValidationResultV2> => {
    const { registry } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        aggregatedAssets,
        sourceParachain: source,
    } = transfer.computed
    const { tx } = transfer

    const { sourceParachain, gateway, ethereum, bridgeHub, assetHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  gateway: context.gateway(),
                  ethereum: context.ethereum(),
                  bridgeHub: await context.bridgeHub(),
                  assetHub: await context.assetHub(),
              }
            : context

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
    let dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    let tokenBalances: ConcreteToken[] = []
    for (let asset of aggregatedAssets) {
        let tokenAmount: bigint = 0n
        if (!isRelaychainLocation(asset.sourceAssetMetadata.location)) {
            // tokenBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
            tokenAmount = await sourceParachainImpl.getTokenBalance(
                sourceAccountHex,
                registry.ethChainId,
                asset.sourceAssetMetadata.token,
                asset.sourceAssetMetadata,
            )
            tokenBalances.push({
                address: asset.sourceAssetMetadata.token,
                amount: tokenAmount,
            })
        }
    }
    let contractCall = transfer.input.contractCall
    if (contractCall) {
        try {
            await checkContractAddress(ethereum, contractCall.target)
        } catch (error) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.ContractCallInvalidTarget,
                message:
                    "Contract call with invalid target address: " +
                    contractCall.target +
                    " error: " +
                    String(error),
            })
        }
    }

    let sourceDryRunError
    let assetHubDryRunError
    let bridgeHubDryRunError
    // do the dry run, get the forwarded xcm and dry run that
    if (sourceParaId == registry.assetHubParaId) {
        const dryRunResultAssetHub = await dryRunOnSourceParachain(
            assetHub,
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
    } else {
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
                        message: "Dry run call did not provide a forwarded xcm.",
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
            tokenBalances,
            sourceDryRunError,
            assetHubDryRunError,
            bridgeHubDryRunError,
        },
        transfer,
    }
}

export async function buildContractCallHex(context: Context, contractCall: ContractCall) {
    const bridgeHub = await context.bridgeHub()
    const callHex = bridgeHub.createType("ContractCall", {
        target: contractCall.target,
        calldata: contractCall.calldata,
        value: contractCall.value,
        gas: contractCall.gas,
    })
    return "0x00" + callHex.toHex().slice(2)
}

export const mockDeliveryFee: DeliveryFeeV2 = {
    localExecutionFeeDOT: 1n,
    snowbridgeDeliveryFeeDOT: 1n,
    assetHubExecutionFeeDOT: 1n,
    bridgeHubDeliveryFeeDOT: 1n,
    totalFeeInDot: 10n,
    ethereumExecutionFee: 1n,
}

export const checkContractAddress = async (ethereum: AbstractProvider, address: string) => {
    if (!ethers.isAddress(address)) {
        throw new Error("Invalid contract address: " + address)
    }
    try {
        const code = await ethereum.getCode(address)
        if (code == "0x") {
            throw new Error(
                "Contract call with invalid target address: no contract deployed at " + address,
            )
        }
    } catch (error) {
        throw new Error(
            "Contract call with invalid target address: " + address + " error: " + String(error),
        )
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
