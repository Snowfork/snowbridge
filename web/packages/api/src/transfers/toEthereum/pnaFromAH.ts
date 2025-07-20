import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { DOT_LOCATION, bridgeLocation, isNative } from "../../xcmBuilder"
import {
    buildExportXcm,
    buildTransferXcmFromAssetHub,
} from "../../xcmbuilders/toEthereum/pnaFromAH"
import { buildTransferXcmFromAssetHubWithDOTAsFee } from "../../xcmbuilders/toEthereum/pnaFromAHWithDotAsFee"
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
    dryRunOnSourceParachain,
    getSnowbridgeDeliveryFee,
    MaxWeight,
} from "../../toEthereumSnowbridgeV2"

export class PNAFromAH implements TransferInterface {
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
        const { tokenErcMetadata, sourceAssetMetadata } = resolveInputs(
            registry,
            tokenAddress,
            sourceParachainImpl.parachainId
        )

        let forwardedXcmToBH, localXcm: any

        let localExecutionFeeDOT = 0n
        let assetHubExecutionFeeDOT = 0n
        let returnToSenderExecutionFeeDOT = 0n
        let returnToSenderDeliveryFeeDOT = 0n
        let bridgeHubDeliveryFeeDOT = 0n

        const assetHubImpl = await paraImplementation(assetHub)
        {
            localXcm = buildTransferXcmFromAssetHub(
                assetHub.registry,
                registry.ethChainId,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000",
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                sourceAssetMetadata,
                1n,
                1n,
                1n
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
        const { ethChainId } = registry

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
            sourceAccount,
            beneficiaryAccount,
            ahAssetMetadata,
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
        if (isNative(transfer.computed.ahAssetMetadata.location)) {
            tokenBalance = await sourceParachainImpl.getNativeBalance(sourceAccountHex)
            isNativeBalance = true
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

        // No fee specified means that the fee.ethereumExecutionFee is paid in Ether on source chain.
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
            },
            transfer,
        }
    }

    createTx(
        parachain: ApiPromise,
        ethChainId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        asset: Asset,
        amount: bigint,
        messageId: string,
        fee: DeliveryFee
    ): SubmittableExtrinsic<"promise", ISubmittableResult> {
        let xcm: any
        // If there is no fee specified, we assume that Ether is available in user's wallet on source chain,
        // thus no swap required on Asset Hub.
        if (!fee.feeLocation) {
            xcm = buildTransferXcmFromAssetHub(
                parachain.registry,
                ethChainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                asset,
                amount,
                fee.totalFeeInDot,
                fee.ethereumExecutionFee!
            )
        } // If the fee asset is in DOT, we need to swap it to Ether on Asset Hub.
        else if (isNative(fee.feeLocation)) {
            xcm = buildTransferXcmFromAssetHubWithDOTAsFee(
                parachain.registry,
                ethChainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                asset,
                amount,
                fee.localExecutionFeeDOT! +
                    fee.bridgeHubDeliveryFeeDOT +
                    fee.snowbridgeDeliveryFeeDOT,
                fee.totalFeeInDot,
                fee.ethereumExecutionFee!
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on AH:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
