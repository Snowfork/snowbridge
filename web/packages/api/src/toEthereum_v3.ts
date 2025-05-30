import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { Codec, ISubmittableResult } from "@polkadot/types/types"
import { BN, hexToU8a, isHex, stringToU8a, u8aToHex } from "@polkadot/util"
import { blake2AsHex, decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import {
    bridgeLocation,
    buildResultXcmAssetHubERC20TransferFromParachain,
    DOT_LOCATION,
    parachainLocation,
    buildParachainERC20ReceivedXcmOnDestination,
    buildResultXcmAssetHubPNATransferFromParachain,
    buildParachainPNAReceivedXcmOnDestination,
    buildExportXcmForPNA,
    buildExportXcmForERC20,
    HERE_LOCATION,
    buildTransferXcmFromAssetHub,
    buildTransferXcmFromParachain,
} from "./xcmV5Builder"
import {
    Asset,
    AssetRegistry,
    calculateDeliveryFee,
    calculateDestinationFee,
    ERC20Metadata,
    getAssetHubConversationPalletSwap,
    getDotBalance,
    getEtherBalance,
    getNativeBalance,
    getParachainId,
    getTokenBalance,
    padFeeByPercentage,
    Parachain,
} from "./assets_v3"
import { getOperatingStatus, OperationStatus } from "./status"
import { IGatewayV1 as IGateway } from "@snowbridge/contract-types"
import {
    CallDryRunEffects,
    EventRecord,
    XcmDryRunApiError,
    XcmDryRunEffects,
} from "@polkadot/types/interfaces"
import { Result } from "@polkadot/types"
import { AbstractProvider, FeeData } from "ethers"

export type Transfer = {
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
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
    totalFeeInNative?: bigint
    assetHubExecutionFeeNative?: bigint
    returnToSenderExecutionFeeNative?: bigint
    ethereumExecutionFee: bigint
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

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

    const sourceParaId = await getParachainId(parachain)
    const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
        resolveInputs(registry, tokenAddress, sourceParaId)

    let messageId: string | undefined = await buildMessageId(
        parachain,
        sourceParaId,
        sourceAccountHex,
        tokenAddress,
        beneficiaryAccount,
        amount
    )
    let tx: SubmittableExtrinsic<"promise", ISubmittableResult>
    if (sourceParaId === assetHubParaId) {
        tx = createAssetHubTx(
            parachain,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            ahAssetMetadata,
            amount,
            DOT_LOCATION,
            fee.assetHubExecutionFeeDOT || 800_000_000_000n,
            bridgeLocation(ethChainId),
            fee.ethereumExecutionFee,
            messageId
        )
    } else {
        const fee_on_ah =
            fee.assetHubExecutionFeeDOT + fee.snowbridgeDeliveryFeeDOT + fee.bridgeHubDeliveryFeeDOT
        tx = createSourceParachainTx(
            parachain,
            ethChainId,
            sourceAccountHex,
            beneficiaryAccount,
            sourceAssetMetadata,
            amount,
            DOT_LOCATION,
            fee.totalFeeInDot - fee_on_ah,
            DOT_LOCATION,
            fee_on_ah,
            bridgeLocation(ethChainId),
            fee.ethereumExecutionFee,
            assetHubParaId,
            sourceParaId,
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
            sourceParaId,
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
    const feePadPercentage = padPercentage ?? 100n
    const feeStorageKey = xxhashAsHex(":BridgeHubEthereumBaseFeeV2:", 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace("0x", ""), "hex", "le")

    let snowbridgeDeliveryFeeDOT = 0n
    if (leFee.eqn(0)) {
        console.warn("Asset Hub onchain BridgeHubEthereumBaseFee not set. Using default fee.")
        snowbridgeDeliveryFeeDOT = defaultFee ?? 200_000_000_000n
    } else {
        snowbridgeDeliveryFeeDOT = BigInt(leFee.toString())
    }

    const { tokenErcMetadata, sourceAssetMetadata, sourceParachain } = resolveInputs(
        registry,
        tokenAddress,
        parachain
    )

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
            340282366920938463463374607431768211455n
        )
        forwardedXcm = buildExportXcmForPNA(
            assetHub.registry,
            registry.ethChainId,
            sourceAssetMetadata.locationOnEthereum,
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            1000
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
                DOT_LOCATION
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
                HERE_LOCATION,
                parachainLocation(sourceParachain.parachainId)
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
            1000
        )
    }

    let assetHubExecutionFeeDOT = 0n
    let returnToSenderExecutionFeeDOT = 0n
    let returnToSenderDeliveryFeeDOT = 0n
    let bridgeHubDeliveryFeeDOT = await calculateDeliveryFee(
        assetHub,
        registry.bridgeHubParaId,
        forwardedXcm
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

        returnToSenderDeliveryFeeDOT = await calculateDeliveryFee(
            assetHub,
            parachain,
            returnToSenderXcm
        )
        if (registry.parachains[parachain].features.hasXcmPaymentApi) {
            returnToSenderExecutionFeeDOT = padFeeByPercentage(
                await calculateDestinationFee(source, returnToSenderXcm),
                feePadPercentage
            )
        } else {
            console.warn(
                `Parachain ${parachain} does not support payment apis. Using a manually estimated fee.`
            )
            returnToSenderExecutionFeeDOT = padFeeByPercentage(
                registry.parachains[parachain].estimatedExecutionFeeDOT,
                feePadPercentage
            )
        }
        assetHubExecutionFeeDOT = padFeeByPercentage(
            await calculateDestinationFee(assetHub, xcm),
            feePadPercentage
        )
    }

    const totalFeeInDot =
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
    }
    transfer: Transfer
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
    const nativeBalance = await getNativeBalance(sourceParachain, sourceAccountHex)
    let dotBalance: bigint | undefined = undefined
    if (source.features.hasDotBalance) {
        dotBalance = await getDotBalance(sourceParachain, source.info.specName, sourceAccountHex)
    }
    let tokenBalance: any
    let isNativeBalance = false
    // For DOT on AH, get it from the native balance pallet.
    if (
        sourceParaId == registry.assetHubParaId &&
        transfer.computed.ahAssetMetadata.location?.parents == DOT_LOCATION.parents &&
        transfer.computed.ahAssetMetadata.location?.interior == DOT_LOCATION.interior
    ) {
        tokenBalance = await getNativeBalance(sourceParachain, sourceAccountHex)
        isNativeBalance = true
    } else {
        tokenBalance = await getTokenBalance(
            sourceParachain,
            source.info.specName,
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata
        )
        isNativeBalance =
            sourceAssetMetadata.decimals === source.info.tokenDecimals &&
            sourceAssetMetadata.symbol == source.info.tokenSymbols
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

    let etherBalance = await getEtherBalance(
        sourceParachain,
        source.info.specName,
        sourceAccountHex,
        registry.ethChainId
    )
    if (fee.ethereumExecutionFee > etherBalance) {
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
                let xcmOnAssetHub = dryRunSource.assetHubForwarded[1][0]
                console.log("forward xcm on AH:", xcmOnAssetHub.toHuman())
                const dryRunResultAssetHub = await dryRunAssetHub(
                    assetHub,
                    sourceParaId,
                    registry.bridgeHubParaId,
                    xcmOnAssetHub
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
    let paymentInfo, sourceExecutionFee
    try {
        paymentInfo = await tx.paymentInfo(sourceAccountHex)
        sourceExecutionFee = paymentInfo["partialFee"].toBigInt()
    } catch {
        sourceExecutionFee = 1_000_000_000n
    }

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
    parachain: ApiPromise,
    transfer: Transfer,
    account: AddressOrPair,
    options: Partial<SignerOptions>
): Promise<MessageReceipt> {
    const result = await new Promise<MessageReceipt>((resolve, reject) => {
        try {
            transfer.tx.signAndSend(account, options, (c) => {
                if (c.isError) {
                    console.error(c)
                    reject(c.internalError || c.dispatchError || c)
                }
                if (c.isInBlock) {
                    const result = {
                        txHash: u8aToHex(c.txHash),
                        txIndex: c.txIndex || 0,
                        blockNumber: Number((c as any).blockNumber),
                        blockHash: "",
                        events: c.events,
                    }
                    for (const e of c.events) {
                        if (parachain.events.system.ExtrinsicFailed.is(e.event)) {
                            resolve({
                                ...result,
                                success: false,
                                dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError,
                            })
                        }

                        if (parachain.events.polkadotXcm.Sent.is(e.event)) {
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

    result.blockHash = u8aToHex(await parachain.rpc.chain.getBlockHash(result.blockNumber))
    result.messageId = transfer.computed.messageId ?? result.messageId

    return result
}

function resolveInputs(registry: AssetRegistry, tokenAddress: string, sourceParaId: number) {
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
    sourceAccount: string,
    beneficiaryAccount: string,
    asset: Asset,
    amount: bigint,
    localFeeAsset: any,
    localFeeAmount: bigint,
    remoteFeeAsset: any,
    remoteFeeAmount: bigint,
    messageId: string
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let xcm = buildTransferXcmFromAssetHub(
        parachain.registry,
        ethChainId,
        sourceAccount,
        beneficiaryAccount,
        asset,
        amount,
        localFeeAsset,
        localFeeAmount,
        remoteFeeAsset,
        remoteFeeAmount,
        messageId
    )
    let maxWeight = { refTime: 100_000_000_000n, proofSize: 4_000_000 }
    console.log("xcm on AH:", xcm.toHuman())
    return parachain.tx.polkadotXcm.execute(xcm, maxWeight)
}

function createSourceParachainTx(
    parachain: ApiPromise,
    ethChainId: number,
    sourceAccount: string,
    beneficiaryAccount: string,
    asset: Asset,
    amount: bigint,
    localFeeAssetId: any,
    localFeeAmount: bigint,
    assethubFeeAssetId: any,
    assethubFeeAmount: bigint,
    remoteFeeAssetId: any,
    remoteFeeAmount: bigint,
    assetHubParaId: number,
    sourceParachainId: number,
    messageId: string
): SubmittableExtrinsic<"promise", ISubmittableResult> {
    let xcm = buildTransferXcmFromParachain(
        parachain.registry,
        ethChainId,
        sourceAccount,
        beneficiaryAccount,
        asset,
        amount,
        localFeeAssetId,
        localFeeAmount,
        assethubFeeAssetId,
        assethubFeeAmount,
        remoteFeeAssetId,
        remoteFeeAmount,
        assetHubParaId,
        sourceParachainId,
        messageId
    )
    let maxWeight = { refTime: 100_000_000_000n, proofSize: 4_000_000 }
    console.log("xcm on source chain:", xcm.toHuman())
    return parachain.tx.polkadotXcm.execute(xcm, maxWeight)
}

async function dryRunOnSourceParachain(
    source: ApiPromise,
    assetHubParaId: number,
    bridgeHubParaId: number,
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>,
    sourceAccount: string
) {
    const origin = { system: { signed: sourceAccount } }
    let result = await source.call.dryRunApi.dryRunCall<
        Result<CallDryRunEffects, XcmDryRunApiError>
    >(origin, tx.inner.toHex(), 5)

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

async function dryRunAssetHub(
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

async function buildMessageId(
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
