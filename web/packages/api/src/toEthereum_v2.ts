import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import {
    bridgeLocation,
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildAssetHubERC20TransferFromParachain,
    DOT_LOCATION,
    erc20Location,
    parachainLocation,
    buildParachainERC20ReceivedXcmOnDestination,
    buildResultXcmAssetHubPNATransferFromParachain,
    buildParachainPNAReceivedXcmOnDestination,
    buildAssetHubPNATransferFromParachain,
    buildExportXcmForPNA,
    buildExportXcmForERC20,
    HERE_LOCATION,
    buildAssetHubERC20TransferFromParachainWithNativeFee,
} from "./xcmBuilder"
import { getOperatingStatus, OperationStatus } from "./status"
import {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    Parachain,
} from "@snowbridge/base-types"
import { IGatewayV1 as IGateway } from "@snowbridge/contract-types"
import {
    CallDryRunEffects,
    EventRecord,
    XcmDryRunApiError,
    XcmDryRunEffects,
} from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { FeeData } from "ethers"
import { paraImplementation } from "./parachains"
import { padFeeByPercentage } from "./utils"
import { Context } from "./index"
import { ParachainBase } from "./parachains/parachainBase"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
        contractCall?: ContractCall
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
        contractCall?: ContractCall
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    snowbridgeDeliveryFeeDOT: bigint
    bridgeHubDeliveryFeeDOT: bigint
    assetHubExecutionFeeDOT: bigint
    returnToSenderExecutionFeeDOT: bigint
    returnToSenderDeliveryFeeDOT: bigint
    totalFeeInDot: bigint
    localExecutionFeeDOT?: bigint
    localDeliveryFeeDOT?: bigint
    ethereumExecutionFee?: bigint
    feeLocation?: any
    totalFeeInNative?: bigint
    assetHubExecutionFeeNative?: bigint
    returnToSenderExecutionFeeNative?: bigint
    localExecutionFeeInNative?: bigint
    localDeliveryFeeInNative?: bigint
    ethereumExecutionFeeInNative?: bigint
    l2BridgeFeeInL1Token?: bigint // Fee for the actual token bridge to L2 is paid in the L1 input token.
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export async function createTransfer(
    source: { sourceParaId: number; context: Context } | { parachain: ApiPromise },
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee,
): Promise<Transfer> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    const { parachain } =
        "sourceParaId" in source
            ? { parachain: await source.context.parachain(source.sourceParaId) }
            : source

    const sourceParachainImpl = await paraImplementation(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParachainImpl.parachainId)

    let messageId: string | undefined
    let tx: SubmittableExtrinsic<"promise", ISubmittableResult>
    if (sourceParachainImpl.parachainId === assetHubParaId) {
        // For PNA from foreign consensus
        if (ahAssetMetadata.location?.parents == 2) {
            tx = createAssetHubTxForPNAFromForeignConsensus(
                parachain,
                ethChainId,
                beneficiaryAccount,
                amount,
                ahAssetMetadata,
            )
        } else {
            tx = createAssetHubTx(
                parachain,
                ethChainId,
                tokenAddress,
                beneficiaryAccount,
                amount,
                ahAssetMetadata,
            )
        }
    } else {
        messageId = await buildMessageId(
            parachain,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            tokenAddress,
            beneficiaryAccount,
            amount,
        )
        if (sourceAssetMetadata.location) {
            tx = createPNASourceParachainTx(
                sourceParachainImpl,
                ethChainId,
                assetHubParaId,
                sourceAssetMetadata,
                beneficiaryAccount,
                amount,
                fee.totalFeeInNative ?? fee.totalFeeInDot,
                messageId,
                fee.totalFeeInNative !== undefined,
            )
        } else {
            tx = createERC20SourceParachainTx(
                sourceParachainImpl,
                ethChainId,
                assetHubParaId,
                sourceAccountHex,
                tokenAddress,
                beneficiaryAccount,
                amount,
                fee.totalFeeInNative ?? fee.totalFeeInDot,
                messageId,
                sourceParachainImpl.parachainId,
                fee.returnToSenderExecutionFeeDOT,
                fee.totalFeeInNative !== undefined,
            )
        }
    }

    return {
        input: {
            registry,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        },
        computed: {
            sourceParaId: sourceParachainImpl.parachainId,
            sourceAccountHex,
            tokenErcMetadata,
            sourceParachain,
            ahAssetMetadata,
            sourceAssetMetadata,
            messageId,
        },
        tx,
    }
}

export async function getDeliveryFee(
    context: Context | { assetHub: ApiPromise; source: ApiPromise },
    parachain: number,
    registry: AssetRegistry,
    tokenAddress: string,
    options?: {
        padPercentage?: bigint
        slippagePadPercentage?: bigint
        defaultFee?: bigint
    },
): Promise<DeliveryFee> {
    const { assetHub, source } =
        context instanceof Context
            ? { assetHub: await context.assetHub(), source: await context.parachain(parachain) }
            : context

    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feePadPercentage = options?.padPercentage ?? 33n
    const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFee:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain BridgeHubEthereumBaseFee not set. Using default fee.")
        snowbridgeDeliveryFeeDOT = options?.defaultFee ?? 3_833_568_200_000n
    } else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }

    const { sourceAssetMetadata, sourceParachain } = resolveInputs(
        registry,
        tokenAddress,
        parachain,
    )
    const sourceParachainImpl = await paraImplementation(source)

    let xcm: any, forwardedXcm: any

    if (sourceAssetMetadata.location) {
        xcm = buildResultXcmAssetHubPNATransferFromParachain(
            assetHub.registry,
            registry.ethChainId,
            sourceAssetMetadata.locationOnAH,
            sourceAssetMetadata.locationOnEthereum,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
        )
        forwardedXcm = buildExportXcmForPNA(
            assetHub.registry,
            registry.ethChainId,
            sourceAssetMetadata.locationOnEthereum,
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            1000,
        )
    } else {
        if (sourceParachain.features.hasDotBalance) {
            xcm = buildResultXcmAssetHubERC20TransferFromParachain(
                assetHub.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                parachain,
                340282366920938463463374607431768211455n,
                DOT_LOCATION,
                DOT_LOCATION,
                false,
            )
        } else {
            xcm = buildResultXcmAssetHubERC20TransferFromParachain(
                assetHub.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                parachain,
                340282366920938463463374607431768211455n,
                sourceParachainImpl.getNativeBalanceLocation("here"),
                sourceParachainImpl.getNativeBalanceLocation("sibling"),
                true,
            )
        }
        forwardedXcm = buildExportXcmForERC20(
            assetHub.registry,
            registry.ethChainId,
            tokenAddress,
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            1000,
        )
    }

    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    const assetHubImpl = await paraImplementation(assetHub)
    const bridgeHubDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
        registry.bridgeHubParaId,
        forwardedXcm,
    )
    if (parachain !== registry.assetHubParaId) {
        let returnToSenderXcm: any
        if (sourceAssetMetadata.location) {
            returnToSenderXcm = buildParachainPNAReceivedXcmOnDestination(
                source.registry,
                sourceAssetMetadata.location,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        } else {
            returnToSenderXcm = buildParachainERC20ReceivedXcmOnDestination(
                source.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            )
        }

        returnToSenderDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            parachain,
            returnToSenderXcm,
        )
        returnToSenderExecutionFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(returnToSenderXcm, DOT_LOCATION),
            feePadPercentage,
        )
        assetHubExecutionFeeDOT = padFeeByPercentage(
            await assetHubImpl.calculateXcmFee(xcm, DOT_LOCATION),
            feePadPercentage,
        )
    }

    let totalFeeInDot =
        snowbridgeDeliveryFeeDOT +
        assetHubExecutionFeeDOT +
        returnToSenderExecutionFeeDOT +
        returnToSenderDeliveryFeeDOT +
        bridgeHubDeliveryFeeDOT

    // calculate the cost of swapping for DOT
    let totalFeeInNative: bigint | undefined = undefined
    let assetHubExecutionFeeNative: bigint | undefined = undefined
    let returnToSenderExecutionFeeNative: bigint | undefined = undefined
    if (!registry.parachains[parachain].features.hasDotBalance) {
        // padding the bridging fee and bridge hub delivery by the slippage fee to make sure the trade goes through.
        totalFeeInDot =
            padFeeByPercentage(
                snowbridgeDeliveryFeeDOT + bridgeHubDeliveryFeeDOT,
                feeSlippagePadPercentage,
            ) +
            assetHubExecutionFeeDOT +
            returnToSenderExecutionFeeDOT +
            returnToSenderDeliveryFeeDOT

        const nativeLocation = sourceParachainImpl.getNativeBalanceLocation("sibling")
        const [
            totalFeeInNativeRes,
            assetHubExecutionFeeNativeRes,
            returnToSenderExecutionFeeNativeRes,
        ] = await Promise.all([
            assetHubImpl.getAssetHubConversionPalletSwap(
                nativeLocation,
                DOT_LOCATION,
                totalFeeInDot,
            ),
            assetHubImpl.getAssetHubConversionPalletSwap(
                nativeLocation,
                DOT_LOCATION,
                assetHubExecutionFeeDOT,
            ),
            assetHubImpl.getAssetHubConversionPalletSwap(
                nativeLocation,
                DOT_LOCATION,
                returnToSenderExecutionFeeDOT,
            ),
        ])
        totalFeeInNative = totalFeeInNativeRes
        assetHubExecutionFeeNative = assetHubExecutionFeeNativeRes
        returnToSenderExecutionFeeNative = returnToSenderExecutionFeeNativeRes
    }

    return {
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot,
        totalFeeInNative,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative,
    }
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    BridgeStatusNotOperational,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientDotFee,
    InsufficientNativeFee,
    DryRunApiNotAvailable,
    DryRunFailed,
    InsufficientEtherBalance,
    ContractCallInvalidTarget,
    ContractCallAgentNotRegistered,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type ValidationResult = {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance?: bigint
        sourceExecutionFee: bigint
        tokenBalance: bigint
        sourceDryRunError: any
        assetHubDryRunError: any
        bridgeHubDryRunError?: any
    }
    transfer: Transfer
}

export async function validateTransfer(
    context:
        | Context
        | {
              sourceParachain: ApiPromise
              assetHub: ApiPromise
              gateway: IGateway
              bridgeHub: ApiPromise
          },
    transfer: Transfer,
): Promise<ValidationResult> {
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        sourceParachain: source,
        sourceAssetMetadata,
    } = transfer.computed

    const { sourceParachain, assetHub, gateway, bridgeHub } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
                  assetHub: await context.assetHub(),
                  gateway: context.gateway(),
                  bridgeHub: await context.bridgeHub(),
              }
            : context

    const { tx } = transfer

    const logs: ValidationLog[] = []
    const sourceParachainImpl = await paraImplementation(sourceParachain)
    const nativeBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
    let dotBalance: bigint | undefined = undefined
    if (source.features.hasDotBalance) {
        dotBalance = await sourceParachainImpl.getDotBalance(sourceAccountHex)
    }
    let tokenBalance: any
    let isNativeBalance = false
    // For DOT on AH, get it from the native balance pallet.
    if (
        sourceParaId == registry.assetHubParaId &&
        transfer.computed.ahAssetMetadata.location?.parents == DOT_LOCATION.parents &&
        transfer.computed.ahAssetMetadata.location?.interior == DOT_LOCATION.interior
    ) {
        tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
        isNativeBalance = true
    } else {
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
    }
    let nativeBalanceCheckFailed = false
    if (isNativeBalance && fee.totalFeeInNative) {
        nativeBalanceCheckFailed = true
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
            if (sourceParaId == registry.assetHubParaId) {
                if (!dryRunSource.bridgeHubForwarded) {
                    logs.push({
                        kind: ValidationKind.Error,
                        reason: ValidationReason.DryRunFailed,
                        message: "Dry run call did not provide a forwarded xcm.",
                    })
                } else {
                    const dryRunResultBridgeHub = await dryRunBridgeHub(
                        bridgeHub,
                        registry.assetHubParaId,
                        dryRunSource.bridgeHubForwarded[1][0],
                    )
                    if (!dryRunResultBridgeHub.success) {
                        logs.push({
                            kind: ValidationKind.Error,
                            reason: ValidationReason.DryRunFailed,
                            message: "Dry run failed on Bridge Hub.",
                        })
                        bridgeHubDryRunError = dryRunResultBridgeHub.errorMessage
                    }
                }
            } else {
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
                            message: "Dry run call failed on Asset Hub.",
                        })
                        assetHubDryRunError = dryRunResultAssetHub.errorMessage
                    }
                }
            }
        }
    } else {
        logs.push({
            kind: ValidationKind.Warning,
            reason: ValidationReason.DryRunApiNotAvailable,
            message: "Source parachain can not dry run call. Cannot verify success.",
        })
        if (sourceParaId !== registry.assetHubParaId) {
            let dryRunResultAssetHub: any
            if (sourceAssetMetadata.location) {
                dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    buildResultXcmAssetHubPNATransferFromParachain(
                        sourceParachain.registry,
                        registry.ethChainId,
                        sourceAssetMetadata.locationOnAH,
                        sourceAssetMetadata.locationOnEthereum,
                        sourceAccountHex,
                        beneficiaryAccount,
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                        amount,
                        fee.totalFeeInDot,
                        fee.assetHubExecutionFeeDOT,
                    ),
                )
            } else {
                if (!source.features.hasDotBalance && fee.totalFeeInNative) {
                    dryRunResultAssetHub = await dryRunAssetHub(
                        assetHub,
                        sourceParaId,
                        registry.bridgeHubParaId,
                        buildResultXcmAssetHubERC20TransferFromParachain(
                            sourceParachain.registry,
                            registry.ethChainId,
                            sourceAccountHex,
                            beneficiaryAccount,
                            tokenAddress,
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                            amount,
                            fee.totalFeeInNative,
                            fee.assetHubExecutionFeeNative ?? 0n,
                            sourceParaId,
                            fee.returnToSenderExecutionFeeNative ?? 0n,
                            sourceParachainImpl.getNativeBalanceLocation("here"),
                            sourceParachainImpl.getNativeBalanceLocation("sibling"),
                            true,
                        ),
                    )
                } else {
                    dryRunResultAssetHub = await dryRunAssetHub(
                        assetHub,
                        sourceParaId,
                        registry.bridgeHubParaId,
                        buildResultXcmAssetHubERC20TransferFromParachain(
                            sourceParachain.registry,
                            registry.ethChainId,
                            sourceAccountHex,
                            beneficiaryAccount,
                            tokenAddress,
                            "0x0000000000000000000000000000000000000000000000000000000000000000",
                            amount,
                            fee.totalFeeInDot,
                            fee.assetHubExecutionFeeDOT,
                            sourceParaId,
                            fee.returnToSenderExecutionFeeDOT,
                            DOT_LOCATION,
                            DOT_LOCATION,
                            false,
                        ),
                    )
                }
            }
            if (!dryRunResultAssetHub.success) {
                logs.push({
                    kind: ValidationKind.Error,
                    reason: ValidationReason.DryRunFailed,
                    message: "Dry run failed on Asset Hub.",
                })
                assetHubDryRunError = dryRunResultAssetHub.errorMessage
            }
        }
    }

    const paymentInfo = await tx.paymentInfo(sourceAccountHex)
    const sourceExecutionFee = paymentInfo["partialFee"].toBigInt()

    // recheck total after fee estimation
    if (isNativeBalance && fee.totalFeeInNative && !nativeBalanceCheckFailed) {
        if (amount + fee.totalFeeInNative + sourceExecutionFee > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }
    }

    if (sourceParaId === registry.assetHubParaId) {
        if (!dotBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: "Could not determine the DOT balance",
            })
        } else if (sourceExecutionFee + fee.totalFeeInDot > dotBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: "Insufficient DOT balance to submit transaction on the source parachain.",
            })
        }
    } else {
        if (dotBalance && fee.totalFeeInDot > dotBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientDotFee,
                message: "Insufficient DOT balance to submit transaction on the source parachain.",
            })
        } else if (
            fee.totalFeeInNative &&
            fee.totalFeeInNative + sourceExecutionFee > nativeBalance &&
            !nativeBalanceCheckFailed
        ) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message:
                    "Insufficient native balance to submit transaction on the source parachain.",
            })
        }
        if (sourceExecutionFee > nativeBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientNativeFee,
                message:
                    "Insufficient native balance to submit transaction on the source parachain.",
            })
        }
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
        },
        transfer,
    }
}

export type MessageReceipt = {
    blockNumber: number
    blockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
}

export async function signAndSend(
    context: Context | { sourceParachain: ApiPromise },
    transfer: Transfer,
    account: AddressOrPair,
    options: Partial<SignerOptions>,
): Promise<MessageReceipt> {
    const { sourceParaId } = transfer.computed
    const { sourceParachain } =
        context instanceof Context
            ? {
                  sourceParachain: await context.parachain(sourceParaId),
              }
            : context

    const result = await new Promise<MessageReceipt>((resolve, reject) => {
        try {
            transfer.tx.signAndSend(account, options, (c) => {
                if (c.isError) {
                    console.error(c)
                    reject(c.internalError || c.dispatchError || c)
                }
                // We have to check for finalization here because re-orgs will produce a different messageId on Asset Hub.
                // TODO: Change back to isInBlock when we switch to pallet-xcm.execute for Asset Hub and we can generate the messageId offchain.
                if (c.isFinalized) {
                    const result = {
                        txHash: u8aToHex(c.txHash),
                        txIndex: c.txIndex || 0,
                        blockNumber: Number((c as any).blockNumber),
                        blockHash: "",
                        events: c.events,
                    }
                    for (const e of c.events) {
                        if (sourceParachain.events.system.ExtrinsicFailed.is(e.event)) {
                            resolve({
                                ...result,
                                success: false,
                                dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError,
                            })
                        }

                        if (sourceParachain.events.polkadotXcm.Sent.is(e.event)) {
                            resolve({
                                ...result,
                                success: true,
                                messageId: (e.event.data.toPrimitive() as any)[3],
                            })
                        }
                    }
                    resolve({
                        ...result,
                        success: false,
                    })
                }
            })
        } catch (e) {
            console.error(e)
            reject(e)
        }
    })

    result.blockHash = u8aToHex(await sourceParachain.rpc.chain.getBlockHash(result.blockNumber))
    result.messageId = transfer.computed.messageId ?? result.messageId

    return result
}

export function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number) {
    const tokenErcMetadata =
        registry.ethereumChains[registry.ethChainId.toString()].assets[tokenAddress.toLowerCase()]
    if (!tokenErcMetadata) {
        throw Error(`No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`)
    }
    const sourceParachain = registry.parachains[sourceParaId.toString()]
    if (!sourceParachain) {
        throw Error(`Could not find ${sourceParaId} in the asset registry.`)
    }
    const ahAssetMetadata =
        registry.parachains[registry.assetHubParaId].assets[tokenAddress.toLowerCase()]
    if (!ahAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on asset hub.`)
    }

    const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
    if (!sourceAssetMetadata) {
        throw Error(`Token ${tokenAddress} not registered on source parachain ${sourceParaId}.`)
    }

    return { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata }
}

function createAssetHubTx(
    parachain: ApiPromise,
    ethChainId: number,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    asset: Asset,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    // Asset with location not null for PNA
    let assetLocation = asset.location || erc20Location(ethChainId, tokenAddress)
    const assets = {
        v4: [
            {
                id: assetLocation,
                fun: { Fungible: amount },
            },
        ],
    }
    const feeAsset = {
        v4: assetLocation,
    }
    const destination = { v4: bridgeLocation(ethChainId) }
    let customXcm = parachain.registry.createType("XcmVersionedXcm", {
        v4: [
            {
                depositAsset: {
                    assets: {
                        Wild: {
                            AllCounted: 1,
                        },
                    },
                    beneficiary: {
                        parents: 0,
                        interior: { x1: [{ accountKey20: { key: beneficiaryAccount } }] },
                    },
                },
            },
        ],
    })
    let reserveType = asset.location ? "LocalReserve" : "DestinationReserve"
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        reserveType,
        feeAsset,
        reserveType,
        customXcm,
        "Unlimited",
    )
}

export function createERC20SourceParachainTx(
    parachain: ParachainBase,
    ethChainId: number,
    assetHubParaId: number,
    sourceAccount: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    totalFee: bigint,
    messageId: string,
    sourceParaId: number,
    returnToSenderFeeInDOT: bigint,
    useNativeAssetAsFee: boolean,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const feeAssetId = useNativeAssetAsFee
        ? parachain.getNativeBalanceLocation("here")
        : DOT_LOCATION
    const assets = {
        v4: [
            {
                id: feeAssetId,
                fun: { Fungible: totalFee },
            },
            {
                id: erc20Location(ethChainId, tokenAddress),
                fun: { Fungible: amount },
            },
        ],
    }
    const destination = { v4: parachainLocation(assetHubParaId) }

    const feeAsset = {
        v4: feeAssetId,
    }
    let customXcm
    if (useNativeAssetAsFee) {
        customXcm = buildAssetHubERC20TransferFromParachainWithNativeFee(
            parachain.provider.registry,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            messageId,
            sourceParaId,
            amount,
            returnToSenderFeeInDOT,
            parachain.getNativeBalanceLocation("sibling"),
        )
    } else {
        customXcm = buildAssetHubERC20TransferFromParachain(
            parachain.provider.registry,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            messageId,
            sourceParaId,
            returnToSenderFeeInDOT,
            feeAssetId,
        )
    }
    return parachain.provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        "DestinationReserve",
        feeAsset,
        useNativeAssetAsFee ? "Teleport" : "DestinationReserve",
        customXcm,
        "Unlimited",
    )
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
        >(origin, tx.inner.toHex(), 4)
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
    const sourceParachain = { v4: { parents: 1, interior: { x1: [{ parachain: parachainId }] } } }
    const result = await assetHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultPrimitive = result.toPrimitive() as any
    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete
    let sourceParachainForwarded
    let bridgeHubForwarded
    if (!success) {
        console.error("Error during dry run on asset hub:", xcm.toHuman(), result.toHuman())
    } else {
        bridgeHubForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === bridgeHubParaId
            )
        })
        sourceParachainForwarded = result.asOk.forwardedXcms.find((x) => {
            return (
                x[0].isV4 &&
                x[0].asV4.parents.toNumber() === 1 &&
                x[0].asV4.interior.isX1 &&
                x[0].asV4.interior.asX1[0].isParachain &&
                x[0].asV4.interior.asX1[0].asParachain.toNumber() === parachainId
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

export async function buildMessageId(
    parachain: ApiPromise,
    sourceParaId: number,
    sourceAccountHex: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint,
    timestamp?: number,
): Promise<string> {
    const [accountNextId] = await Promise.all([
        parachain.rpc.system.accountNextIndex(sourceAccountHex),
    ])
    const entropy = new Uint8Array([
        ...stringToU8a(sourceParaId.toString()),
        ...hexToU8a(sourceAccountHex),
        ...accountNextId.toU8a(),
        ...hexToU8a(tokenAddress),
        ...stringToU8a(beneficiaryAccount),
        ...stringToU8a(amount.toString()),
        ...stringToU8a((timestamp || Date.now()).toString()),
    ])
    return blake2AsHex(entropy)
}

function createPNASourceParachainTx(
    parachain: ParachainBase,
    ethChainId: number,
    assetHubParaId: number,
    asset: Asset,
    beneficiaryAccount: string,
    amount: bigint,
    totalFee: bigint,
    messageId: string,
    useNativeAssetAsFee: boolean,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const feeAssetId = useNativeAssetAsFee
        ? parachain.getNativeBalanceLocation("here")
        : DOT_LOCATION
    const assets = {
        v4: [
            {
                id: asset.location,
                fun: { Fungible: amount },
            },
            {
                id: feeAssetId,
                fun: { Fungible: totalFee },
            },
        ],
    }
    const destination = { v4: parachainLocation(assetHubParaId) }

    const feeAsset = {
        v4: feeAssetId,
    }
    const customXcm = buildAssetHubPNATransferFromParachain(
        parachain.provider.registry,
        ethChainId,
        beneficiaryAccount,
        asset.locationOnAH,
        asset.locationOnEthereum,
        messageId,
    )

    return parachain.provider.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        "Teleport",
        feeAsset,
        useNativeAssetAsFee ? "Teleport" : "DestinationReserve",
        customXcm,
        "Unlimited",
    )
}

function createAssetHubTxForPNAFromForeignConsensus(
    parachain: ApiPromise,
    ethChainId: number,
    beneficiaryAccount: string,
    amount: bigint,
    asset: Asset,
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    const assets = {
        v4: [
            {
                id: asset.location,
                fun: { Fungible: amount },
            },
        ],
    }
    const feeAsset = {
        v4: asset.location,
    }
    const destination = { v4: bridgeLocation(ethChainId) }
    let customXcm = parachain.registry.createType("XcmVersionedXcm", {
        v4: [
            {
                depositAsset: {
                    assets: {
                        Wild: {
                            AllCounted: 1,
                        },
                    },
                    beneficiary: {
                        parents: 0,
                        interior: { x1: [{ accountKey20: { key: beneficiaryAccount } }] },
                    },
                },
            },
        ],
    })
    return parachain.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
        destination,
        assets,
        "LocalReserve",
        feeAsset,
        "LocalReserve",
        customXcm,
        "Unlimited",
    )
}

export async function dryRunBridgeHub(bridgeHub: ApiPromise, assetHubParaId: number, xcm: any) {
    const sourceParachain = {
        v5: { parents: 1, interior: { x1: [{ parachain: assetHubParaId }] } },
    }
    const result = await bridgeHub.call.dryRunApi.dryRunXcm<
        Result<XcmDryRunEffects, XcmDryRunApiError>
    >(sourceParachain, xcm)

    const resultHuman = result.toHuman() as any

    const success = result.isOk && result.asOk.executionResult.isComplete

    if (!success) {
        console.error("Error during dry run on bridge hub:", xcm.toHuman(), result.toHuman())
    }
    return {
        success,
        errorMessage: resultHuman.Ok.executionResult.Incomplete?.error,
    }
}
