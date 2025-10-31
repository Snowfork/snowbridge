import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { isRelaychainLocation } from "../../xcmBuilder"
import {
    buildExportXcm,
    buildTransferXcmFromAssetHub,
} from "../../xcmbuilders/toEthereum/erc20FromAH"
import { buildTransferXcmFromAssetHubWithDOTAsFee } from "../../xcmbuilders/toEthereum/erc20FromAHWithDotAsFee"
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
    estimateFeesFromAssetHub,
    MaxWeight,
    validateTransferFromAssetHub,
} from "../../toEthereumSnowbridgeV2"

export class ERC20FromAH implements TransferInterface {
    async getDeliveryFee(
        source: { sourceParaId: number; context: Context },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
        },
    ): Promise<DeliveryFee> {
        const { assetHub } =
            "sourceParaId" in source
                ? {
                      assetHub: await source.context.assetHub(),
                  }
                : source

        const { sourceAssetMetadata } = resolveInputs(registry, tokenAddress, source.sourceParaId)

        let localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            1n,
            1n,
        )

        let forwardedXcmToBH = buildExportXcm(
            assetHub.registry,
            registry.ethChainId,
            sourceAssetMetadata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            1n,
            1n,
        )

        const fees = await estimateFeesFromAssetHub(
            source.context,
            registry,
            tokenAddress,
            {
                localXcm,
                forwardedXcmToBH,
            },
            options,
        )
        return fees
    }

    async createTransfer(
        source: { sourceParaId: number; context: Context },
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
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
            amount,
        )
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> = this.createTx(
            parachain,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            ahAssetMetadata,
            amount,
            messageId,
            fee,
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
        return validateTransferFromAssetHub(context, transfer)
    }

    createTx(
        parachain: ApiPromise,
        ethChainId: number,
        sourceAccount: string,
        beneficiaryAccount: string,
        asset: Asset,
        amount: bigint,
        messageId: string,
        fee: DeliveryFee,
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
                fee.ethereumExecutionFee!,
            )
        } // If the fee asset is in DOT, we need to swap it to Ether on Asset Hub.
        else if (isRelaychainLocation(fee.feeLocation)) {
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
                fee.ethereumExecutionFee!,
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on AH:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
