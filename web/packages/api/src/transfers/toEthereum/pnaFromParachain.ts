import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { isRelaychainLocation, isParachainNative } from "../../xcmBuilder"
import { buildExportXcm } from "../../xcmbuilders/toEthereum/pnaFromAH"
import {
    buildResultXcmAssetHubPNATransferFromParachain,
    buildParachainPNAReceivedXcmOnDestination,
    buildTransferXcmFromParachain,
} from "../../xcmbuilders/toEthereum/pnaFromParachain"
import { buildTransferXcmFromParachainWithDOTAsFee } from "../../xcmbuilders/toEthereum/pnaFromParachainWithDotAsFee"
import { buildTransferXcmFromParachainWithNativeAssetFee } from "../../xcmbuilders/toEthereum/pnaFromParachainWithNativeAsFee"
import { Asset, AssetRegistry } from "@snowbridge/base-types"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
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
    estimateFeesFromParachains,
    MaxWeight,
} from "../../toEthereumSnowbridgeV2"

export class PNAFromParachain implements TransferInterface {
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
        const { assetHub, parachain } =
            "sourceParaId" in source
                ? {
                      assetHub: await source.context.assetHub(),
                      parachain: await source.context.parachain(source.sourceParaId),
                  }
                : source

        const sourceParachainImpl = await paraImplementation(parachain)
        const { sourceAssetMetadata } = resolveInputs(
            registry,
            tokenAddress,
            sourceParachainImpl.parachainId
        )

        let forwardXcmToAH: any, forwardedXcmToBH: any, returnToSenderXcm: any, localXcm: any

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

        returnToSenderXcm = buildParachainPNAReceivedXcmOnDestination(
            parachain.registry,
            sourceAssetMetadata.location,
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
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

        const fees = await estimateFeesFromParachains(
            source.context,
            source.sourceParaId,
            registry,
            tokenAddress,
            {
                localXcm,
                forwardXcmToAH,
                forwardedXcmToBH,
                returnToSenderXcm,
            },
            options
        )
        return fees
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
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult>
        tx = this.createTx(
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
                sourceAssetMetadata
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
                fee.localExecutionFeeDOT! +
                    fee.localDeliveryFeeDOT! +
                    fee.returnToSenderExecutionFeeDOT,
                fee.totalFeeInDot,
                fee.ethereumExecutionFee!
            )
        } // One swap from DOT to Ether on Asset Hub.
        else if (isRelaychainLocation(fee.feeLocation)) {
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
                fee.localExecutionFeeDOT! +
                    fee.localDeliveryFeeDOT! +
                    fee.returnToSenderExecutionFeeDOT,
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
                fee.localExecutionFeeInNative! +
                    fee.localDeliveryFeeInNative! +
                    fee.returnToSenderExecutionFeeNative!,
                fee.totalFeeInNative!,
                fee.ethereumExecutionFee!,
                fee.ethereumExecutionFeeInNative!
            )
        } else {
            throw new Error(
                `Fee token as ${fee.feeLocation} is not supported. Only DOT or native asset is allowed.`
            )
        }
        console.log("xcm on source chain:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
