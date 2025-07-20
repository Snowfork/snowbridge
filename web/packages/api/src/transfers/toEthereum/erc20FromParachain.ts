import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import {
    DOT_LOCATION,
    parachainLocation,
    HERE_LOCATION,
    bridgeLocation,
    isNative,
    isParachainNative,
} from "../../xcmBuilder"
import { buildExportXcm } from "../../xcmbuilders/toEthereum/erc20FromAH"
import {
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildParachainERC20ReceivedXcmOnDestination,
    buildTransferXcmFromParachain,
} from "../../xcmbuilders/toEthereum/erc20FromParachain"
import { buildTransferXcmFromParachainWithDOTAsFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithDotAsFee"
import { buildTransferXcmFromParachainWithNativeAssetFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithNativeAsFee"
import { Asset, AssetRegistry } from "@snowbridge/base-types"
import { ETHER_TOKEN_ADDRESS, getAssetHubConversionPalletSwap } from "../../assets_v2"
import { padFeeByPercentage } from "../../utils"
import { getOperatingStatus } from "../../status"
import { paraImplementation } from "../../parachains"
import {
    buildMessageId,
    DeliveryFee,
    resolveInputs,
    Transfer,
    ValidationKind,
    ValidationLog,
    ValidationReason,
    ValidationResult,
} from "../../toEthereum_v2"
import { Context } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    dryRunAssetHub,
    dryRunOnSourceParachain,
    getSnowbridgeDeliveryFee,
    MaxWeight,
} from "../../toEthereumSnowbridgeV2"

export class ERC20FromParachain implements TransferInterface {
    async getDeliveryFee(
        source: { sourceParaId: number; context: Context },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
        }
    ): Promise<DeliveryFee> {
        const { assetHub, parachain, ethereum } =
            "sourceParaId" in source
                ? {
                      assetHub: await source.context.assetHub(),
                      parachain: await source.context.parachain(source.sourceParaId),
                      ethereum: source.context.ethereum(),
                  }
                : source

        const feePadPercentage = options?.padPercentage ?? 33n
        const feeSlippagePadPercentage = options?.slippagePadPercentage ?? 20n
        const snowbridgeDeliveryFeeDOT = await getSnowbridgeDeliveryFee(
            assetHub,
            options?.defaultFee
        )

        const sourceParachainImpl = await paraImplementation(parachain)
        const { tokenErcMetadata, sourceAssetMetadata, sourceParachain } = resolveInputs(
            registry,
            tokenAddress,
            source.sourceParaId
        )

        let forwardXcmToAH: any, forwardedXcmToBH: any, returnToSenderXcm: any, localXcm: any
        let localExecutionFeeDOT = 0n
        let assetHubExecutionFeeDOT = 0n
        let returnToSenderExecutionFeeDOT = 0n
        let returnToSenderDeliveryFeeDOT = 0n
        let bridgeHubDeliveryFeeDOT = 0n

        const assetHubImpl = await paraImplementation(assetHub)

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
                sourceParachainImpl.parachainId,
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
                sourceParachainImpl.parachainId,
                340282366920938463463374607431768211455n,
                HERE_LOCATION,
                parachainLocation(sourceParachain.parachainId)
            )
        }

        returnToSenderXcm = buildParachainERC20ReceivedXcmOnDestination(
            parachain.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )

        returnToSenderDeliveryFeeDOT = await assetHubImpl.calculateDeliveryFeeInDOT(
            sourceParachainImpl.parachainId,
            returnToSenderXcm
        )
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
            sourceParachainImpl.parachainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            1n,
            10n,
            1n
        )

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
            await assetHubImpl.calculateDeliveryFeeInDOT(
                registry.bridgeHubParaId,
                forwardedXcmToBH
            ),
            feePadPercentage
        )

        let totalFeeInDot =
            localExecutionFeeDOT +
            snowbridgeDeliveryFeeDOT +
            assetHubExecutionFeeDOT +
            returnToSenderExecutionFeeDOT +
            returnToSenderDeliveryFeeDOT +
            bridgeHubDeliveryFeeDOT

        // Calculate execution cost on ethereum
        let ethereumChain = registry.ethereumChains[registry.ethChainId.toString()]
        let feeData = await ethereum.getFeeData()
        let ethereumExecutionFee =
            (feeData.gasPrice ?? 2_000_000_000n) *
            ((tokenErcMetadata.deliveryGas ?? 80_000n) +
                (ethereumChain.baseDeliveryGas ?? 120_000n))

        // calculate the cost of swapping in native asset
        let totalFeeInNative: bigint | undefined = undefined
        let assetHubExecutionFeeNative: bigint | undefined = undefined
        let returnToSenderExecutionFeeNative: bigint | undefined = undefined
        let ethereumExecutionFeeInNative: bigint | undefined
        let localExecutionFeeInNative: bigint | undefined
        let feeLocation = options?.feeTokenLocation
        if (feeLocation) {
            // If the fee asset is DOT, then one swap from DOT to Ether is required on AH
            if (isNative(feeLocation)) {
                ethereumExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                    assetHub,
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage)
                )
                totalFeeInDot += ethereumExecutionFeeInNative
                totalFeeInNative = totalFeeInDot
            }
            // On Parachains, we can use their native asset as the fee token.
            // If the fee is in native, we need to swap it to DOT first, then swap DOT to Ether to cover the ethereum execution fee.
            else {
                localExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                    assetHub,
                    feeLocation,
                    DOT_LOCATION,
                    localExecutionFeeDOT
                )
                returnToSenderExecutionFeeNative = await getAssetHubConversionPalletSwap(
                    assetHub,
                    feeLocation,
                    DOT_LOCATION,
                    returnToSenderExecutionFeeDOT
                )
                let ethereumExecutionFeeInDOT = await getAssetHubConversionPalletSwap(
                    assetHub,
                    DOT_LOCATION,
                    bridgeLocation(registry.ethChainId),
                    padFeeByPercentage(ethereumExecutionFee, feeSlippagePadPercentage)
                )
                ethereumExecutionFeeInNative = await getAssetHubConversionPalletSwap(
                    assetHub,
                    feeLocation,
                    DOT_LOCATION,
                    padFeeByPercentage(ethereumExecutionFeeInDOT, feeSlippagePadPercentage)
                )
                totalFeeInDot += ethereumExecutionFeeInDOT
                totalFeeInNative = await getAssetHubConversionPalletSwap(
                    assetHub,
                    feeLocation,
                    DOT_LOCATION,
                    padFeeByPercentage(totalFeeInDot, feeSlippagePadPercentage)
                )
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

    async createTransfer(
        source: { sourceParaId: number; context: Context } | { parachain: ApiPromise },
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
        const { parachain } =
            "sourceParaId" in source
                ? { parachain: await source.context.parachain(source.sourceParaId) }
                : source

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
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> = this.createTx(
            parachain,
            ethChainId,
            assetHubParaId,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            beneficiaryAccount,
            sourceAssetMetadata,
            amount,
            messageId,
            fee
        )

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

    async validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult> {
        const { registry, fee, tokenAddress, amount, beneficiaryAccount } = transfer.input
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
        tokenBalance = await sourceParachainImpl.getTokenBalance(
            sourceAccountHex,
            registry.ethChainId,
            tokenAddress,
            sourceAssetMetadata
        )
        if (amount > tokenBalance) {
            logs.push({
                kind: ValidationKind.Error,
                reason: ValidationReason.InsufficientTokenBalance,
                message: "Insufficient token balance to submit transaction.",
            })
        }

        if (!fee.feeLocation) {
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
            },
            transfer,
        }
    }

    createTx(
        parachain: ApiPromise,
        ethChainId: number,
        assetHubParaId: number,
        sourceParachainId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        asset: Asset,
        amount: bigint,
        messageId: string,
        fee: DeliveryFee
    ): SubmittableExtrinsic<"promise", ISubmittableResult> {
        let xcm: any
        // No swap
        if (!fee.feeLocation) {
            xcm = buildTransferXcmFromParachain(
                parachain.registry,
                ethChainId,
                assetHubParaId,
                sourceParachainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                asset,
                amount,
                fee.localExecutionFeeDOT! + fee.returnToSenderExecutionFeeDOT,
                fee.totalFeeInDot,
                fee.ethereumExecutionFee!
            )
        } // One swap from DOT to Ether on Asset Hub.
        else if (isNative(fee.feeLocation)) {
            xcm = buildTransferXcmFromParachainWithDOTAsFee(
                parachain.registry,
                ethChainId,
                assetHubParaId,
                sourceParachainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                asset,
                amount,
                fee.localExecutionFeeDOT! + fee.returnToSenderExecutionFeeDOT,
                fee.totalFeeInDot,
                fee.ethereumExecutionFee!,
                fee.ethereumExecutionFeeInNative!
            )
        }
        // If the fee asset is in native asset, we need to swap it to DOT first, then a second swap from DOT to Ether
        else if (isParachainNative(fee.feeLocation, sourceParachainId)) {
            xcm = buildTransferXcmFromParachainWithNativeAssetFee(
                parachain.registry,
                ethChainId,
                assetHubParaId,
                sourceParachainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                asset,
                amount,
                fee.localExecutionFeeInNative! + fee.returnToSenderExecutionFeeNative!,
                fee.totalFeeInNative!,
                fee.ethereumExecutionFee!,
                fee.ethereumExecutionFeeInNative!
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on source chain:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
