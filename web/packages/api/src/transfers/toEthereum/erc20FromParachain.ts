import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { DOT_LOCATION, isRelaychainLocation, isParachainNative } from "../../xcmBuilder"
import { buildExportXcm } from "../../xcmbuilders/toEthereum/erc20FromAH"
import {
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildParachainERC20ReceivedXcmOnDestination,
    buildTransferXcmFromParachain,
} from "../../xcmbuilders/toEthereum/erc20FromParachain"
import { buildTransferXcmFromParachainWithDOTAsFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithDotAsFee"
import { buildTransferXcmFromParachainWithNativeAssetFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithNativeAsFee"
import { Asset, AssetRegistry } from "@snowbridge/base-types"
import { paraImplementation } from "../../parachains"
import {
    buildMessageId,
    DeliveryFee,
    resolveInputs,
    Transfer,
    ValidationResult,
} from "../../toEthereum_v2"
import { Context } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    estimateFeesFromParachains,
    MaxWeight,
    validateTransferFromParachain,
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
            claimerLocation?: any
            accelerated?: boolean
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
        const { sourceAssetMetadata } = resolveInputs(registry, tokenAddress, source.sourceParaId)

        let forwardXcmToAH: any, forwardedXcmToBH: any, returnToSenderXcm: any, localXcm: any

        forwardXcmToAH = buildResultXcmAssetHubERC20TransferFromParachain(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            1n,
            1n,
            1n,
            sourceParachainImpl.parachainId,
            1n,
            DOT_LOCATION,
            DOT_LOCATION
        )

        returnToSenderXcm = buildParachainERC20ReceivedXcmOnDestination(
            parachain.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000",
            1n,
            1n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )

        localXcm = buildTransferXcmFromParachain(
            assetHub.registry,
            registry.environment,
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
        fee: DeliveryFee,
        claimerLocation?: any
    ): Promise<Transfer> {
        const { ethChainId, assetHubParaId, environment } = registry

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
            environment,
            ethChainId,
            assetHubParaId,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            beneficiaryAccount,
            sourceAssetMetadata,
            amount,
            messageId,
            fee,
            claimerLocation
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
        return validateTransferFromParachain(context, transfer)
    }

    createTx(
        parachain: ApiPromise,
        envName: string,
        ethChainId: number,
        assetHubParaId: number,
        sourceParachainId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        asset: Asset,
        amount: bigint,
        messageId: string,
        fee: DeliveryFee,
        claimerLocation?: any
    ): SubmittableExtrinsic<"promise", ISubmittableResult> {
        let xcm: any
        // No swap
        if (!fee.feeLocation) {
            xcm = buildTransferXcmFromParachain(
                parachain.registry,
                envName,
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
                claimerLocation
            )
        } // One swap from DOT to Ether on Asset Hub.
        else if (isRelaychainLocation(fee.feeLocation)) {
            xcm = buildTransferXcmFromParachainWithDOTAsFee(
                parachain.registry,
                envName,
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
                fee.ethereumExecutionFeeInNative!,
                claimerLocation
            )
        }
        // If the fee asset is in native asset, we need to swap it to DOT first, then a second swap from DOT to Ether
        else if (isParachainNative(fee.feeLocation, sourceParachainId)) {
            xcm = buildTransferXcmFromParachainWithNativeAssetFee(
                parachain.registry,
                envName,
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
                fee.ethereumExecutionFeeInNative!,
                claimerLocation
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on source chain:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
