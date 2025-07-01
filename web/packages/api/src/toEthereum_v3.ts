import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import { DOT_LOCATION, parachainLocation, HERE_LOCATION } from "./xcmBuilder"

import {
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildParachainERC20ReceivedXcmOnDestination,
    buildResultXcmAssetHubPNATransferFromParachain,
    buildParachainPNAReceivedXcmOnDestination,
    buildExportXcm,
    buildTransferXcmFromParachain,
    buildTransferXcmFromAssetHub,
} from "./xcmV5Builder"

import {
    Asset,
    AssetRegistry,
    ETHER_TOKEN_ADDRESS,
    getAssetHubConversationPalletSwap,
} from "./assets_v2"
import { padFeeByPercentage } from "./utils"
import { getOperatingStatus } from "./status"
import { IGatewayV1 as IGateway } from "@snowbridge/contract-types"
import { CallDryRunEffects, XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { paraImplementation } from "./parachains"
import {
    DeliveryFee,
    resolveInputs,
    Transfer,
    ValidationKind,
    ValidationLog,
    ValidationReason,
    ValidationResult,
} from "./toEthereum_v2"
import { AbstractProvider } from "ethers"

export { ValidationKind, signAndSend } from "./toEthereum_v2"

export async function createTransfer(
    parachain: ApiPromise,
    registry: AssetRegistry,
    sourceAccount: string,
    beneficiaryAccount: string,
    tokenAddress: string,
    amount: bigint,
    fee: DeliveryFee
): Promise<Transfer> {
    const { ethChainId, assetHubParaId } = registry

    let sourceAccountHex = sourceAccount
    if (!isHex(sourceAccountHex)) {
        sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
    }

    const sourceParachainImpl = await paraImplementation(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParachainImpl.parachainId)

    let messageId: string | undefined = await buildMessageId(
        parachain,
        sourceParachainImpl.parachainId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount
    )
    let tx: SubmittableExtrinsic<"promise", ISubmittableResult>
    if (sourceParachainImpl.parachainId === assetHubParaId) {
        tx = createAssetHubTx(
            parachain,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            ahAssetMetadata,
            amount,
            fee.totalFeeInDot,
            fee.ethereumExecutionFee!,
            messageId
        )
    } else {
        tx = createSourceParachainTx(
            parachain,
            ethChainId,
            assetHubParaId,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            beneficiaryAccount,
            sourceAssetMetadata,
            amount,
            fee.localExecutionFeeDOT! + fee.returnToSenderDeliveryFeeDOT,
            fee.totalFeeInDot,
            fee.assetHubExecutionFeeDOT +
                fee.snowbridgeDeliveryFeeDOT +
                fee.bridgeHubDeliveryFeeDOT,
            fee.ethereumExecutionFee!,
            messageId
        )
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
    connections: { assetHub: ApiPromise; source: ApiPromise; ethereum: AbstractProvider },
    parachain: number,
    registry: AssetRegistry,
    tokenAddress: string,
    padPercentage?: bigint,
    defaultFee?: bigint
): Promise<DeliveryFee> {
    const { assetHub, source, ethereum } = connections
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feePadPercentage = padPercentage ?? 33n
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFeeV2:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain BridgeHubEthereumBaseFee not set. Using default fee.")
        snowbridgeDeliveryFeeDOT = defaultFee ?? 150_000_000_000n
    } else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }

    const { tokenErcMetadata, sourceAssetMetadata, sourceParachain } = resolveInputs(
        registry,
        tokenAddress,
        parachain
    )

    let forwardXcmToAH: any, forwardedXcmToBH: any, returnToSenderXcm: any, localXcm: any

    let localExecutionFeeDOT = 0n
    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = 0n

    const assetHubImpl = await paraImplementation(assetHub)
    if (parachain !== registry.assetHubParaId) {
        if (sourceAssetMetadata.location) {
            forwardXcmToAH = buildResultXcmAssetHubPNATransferFromParachain(
                assetHub.registry,
                registry.ethChainId,
                sourceAssetMetadata.locationOnAH,
                sourceAssetMetadata.locationOnEthereum,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n
            )
        } else {
            if (sourceParachain.features.hasDotBalance) {
                forwardXcmToAH = buildResultXcmAssetHubERC20TransferFromParachain(
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
                    DOT_LOCATION
                )
            } else {
                forwardXcmToAH = buildResultXcmAssetHubERC20TransferFromParachain(
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
                    HERE_LOCATION,
                    parachainLocation(sourceParachain.parachainId)
                )
            }
        }
        if (sourceAssetMetadata.location) {
            returnToSenderXcm = buildParachainPNAReceivedXcmOnDestination(
                source.registry,
                sourceAssetMetadata.location,
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        } else {
            returnToSenderXcm = buildParachainERC20ReceivedXcmOnDestination(
                source.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000",
                340282366920938463463374607431768211455n,
                340282366920938463463374607431768211455n,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            )
        }

        returnToSenderDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            parachain,
            returnToSenderXcm
        )
        const sourceParachainImpl = await paraImplementation(source)
        returnToSenderExecutionFeeDOT = padFeeByPercentage(
            await sourceParachainImpl.calculateXcmFee(returnToSenderXcm, DOT_LOCATION),
            feePadPercentage
        )
        assetHubExecutionFeeDOT = padFeeByPercentage(
            await assetHubImpl.calculateXcmFee(forwardXcmToAH, DOT_LOCATION),
            feePadPercentage
        )
        localXcm = buildTransferXcmFromParachain(
            assetHub.registry,
            registry.ethChainId,
            registry.assetHubParaId,
            parachain,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            1n,
            1n,
            1n,
            1n,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
    } else {
        localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            1n,
            1n,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
    }

    forwardedXcmToBH = buildExportXcm(
        assetHub.registry,
        registry.ethChainId,
        sourceAssetMetadata,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        1n,
        1n
    )

    localExecutionFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateXcmFee(localXcm, DOT_LOCATION),
        feePadPercentage
    )

    bridgeHubDeliveryFeeDOT = padFeeByPercentage(
        await assetHubImpl.calculateDeliveryFeeInDOT(registry.bridgeHubParaId, forwardedXcmToBH),
        feePadPercentage
    )

    const totalFeeInDot =
        localExecutionFeeDOT +
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
        const paraLoc = parachainLocation(parachain)
        const [
            totalFeeInNativeRes,
            assetHubExecutionFeeNativeRes,
            returnToSenderExecutionFeeNativeRes,
        ] = await Promise.all([
            getAssetHubConversationPalletSwap(assetHub, paraLoc, DOT_LOCATION, totalFeeInDot),
            getAssetHubConversationPalletSwap(
                assetHub,
                paraLoc,
                DOT_LOCATION,
                assetHubExecutionFeeDOT
            ),
            getAssetHubConversationPalletSwap(
                assetHub,
                paraLoc,
                DOT_LOCATION,
                returnToSenderExecutionFeeDOT
            ),
        ])
        totalFeeInNative = totalFeeInNativeRes
        assetHubExecutionFeeNative = assetHubExecutionFeeNativeRes
        returnToSenderExecutionFeeNative = returnToSenderExecutionFeeNativeRes
    }

    // Calculate execution cost on ethereum
    let ethereumChain = registry.ethereumChains[registry.ethChainId.toString()]
    let feeData = await ethereum.getFeeData()
    let ethereumExecutionFee =
        (feeData.gasPrice ?? 2_000_000_000n) *
        ((tokenErcMetadata.deliveryGas ?? 100_000n) + (ethereumChain.baseDeliveryGas ?? 180_000n))

    return {
        localExecutionFeeDOT,
        snowbridgeDeliveryFeeDOT,
        assetHubExecutionFeeDOT,
        bridgeHubDeliveryFeeDOT,
        returnToSenderDeliveryFeeDOT,
        returnToSenderExecutionFeeDOT,
        totalFeeInDot,
        totalFeeInNative,
        assetHubExecutionFeeNative,
        returnToSenderExecutionFeeNative,
        ethereumExecutionFee,
    }
}

export async function validateTransfer(
    connections: {
        sourceParachain: ApiPromise
        assetHub: ApiPromise
        gateway: IGateway
        bridgeHub: ApiPromise
    },
    transfer: Transfer
): Promise<ValidationResult> {
    const { sourceParachain, gateway, bridgeHub, assetHub } = connections
    const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
    const {
        sourceAccountHex,
        sourceParaId,
        sourceParachain: source,
        sourceAssetMetadata,
    } = transfer.computed
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
                sourceAssetMetadata
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

    let etherBalance = await sourceParachainImpl.getTokenBalance(
        sourceAccountHex,
        registry.ethChainId,
        ETHER_TOKEN_ADDRESS
    )

    if (fee.ethereumExecutionFee! > etherBalance) {
        logs.push({
            kind: ValidationKind.Error,
            reason: ValidationReason.InsufficientEtherBalance,
            message: "Insufficient ether balance to submit transaction.",
        })
    }

    let sourceDryRunError
    let assetHubDryRunError
    if (source.features.hasDryRunApi) {
        // do the dry run, get the forwarded xcm and dry run that
        const dryRunSource = await dryRunOnSourceParachain(
            sourceParachain,
            registry.assetHubParaId,
            registry.bridgeHubParaId,
            transfer.tx,
            sourceAccountHex
        )
        if (!dryRunSource.success) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.DryRunFailed,
                message: "Dry run call on source failed.",
            })
            sourceDryRunError = dryRunSource.error
        }

        if (dryRunSource.success && sourceParaId !== registry.assetHubParaId) {
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
                    dryRunSource.assetHubForwarded[1][0]
                )
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
                        fee.assetHubExecutionFeeDOT
                    )
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
                            HERE_LOCATION,
                            parachainLocation(sourceParaId)
                        )
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
                            DOT_LOCATION
                        )
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

export async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string
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
            result.toHuman()
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
    xcm: any
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

export async function buildMessageId(
    parachain: ApiPromise,
    sourceParaId: number,
    sourceAccountHex: string,
    tokenAddress: string,
    beneficiaryAccount: string,
    amount: bigint
) {
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
    ])
    return blake2AsHex(entropy)
}

const MaxWeight = { refTime: 15_000_000_000n, proofSize: 800_000 }

function createAssetHubTx(
    parachain: ApiPromise,
    ethChainId: number,
    sourceAccount: string,
    beneficiaryAccount: string,
    asset: Asset,
    amount: bigint,
    totalDOTFeeAmount: bigint,
    remoteEtherFeeAmount: bigint,
    messageId: string
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let xcm = buildTransferXcmFromAssetHub(
        parachain.registry,
        ethChainId,
        sourceAccount,
        beneficiaryAccount,
        asset,
        amount,
        totalDOTFeeAmount,
        remoteEtherFeeAmount,
        messageId
    )
    console.log("xcm on AH:", xcm.toHuman())
    return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
}

function createSourceParachainTx(
    parachain: ApiPromise,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiaryAccount: string,
    asset: Asset,
    amount: bigint,
    localDOTFeeAmount: bigint,
    totalDOTFeeAmount: bigint,
    assethubDOTFeeAmount: bigint,
    remoteEtherFeeAmount: bigint,
    messageId: string
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let xcm = buildTransferXcmFromParachain(
        parachain.registry,
        ethChainId,
        assetHubParaId,
        sourceParachainId,
        sourceAccount,
        beneficiaryAccount,
        asset,
        amount,
        localDOTFeeAmount,
        totalDOTFeeAmount,
        assethubDOTFeeAmount,
        remoteEtherFeeAmount,
        messageId
    )
    console.log("xcm on source chain:", xcm.toHuman())
    return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
}
