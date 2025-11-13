import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { isRelaychainLocation, isParachainNative } from "../../xcmBuilder"
import { buildExportXcm } from "../../xcmbuilders/toEthereum/pnaFromAH"
import {
    buildResultXcmAssetHubPNATransferFromParachain,
    buildTransferXcmFromParachain,
} from "../../xcmbuilders/toEthereum/pnaFromParachain"
import { buildTransferXcmFromParachainWithDOTAsFee } from "../../xcmbuilders/toEthereum/pnaFromParachainWithDotAsFee"
import { buildTransferXcmFromParachainWithNativeAssetFee } from "../../xcmbuilders/toEthereum/pnaFromParachainWithNativeAsFee"
import { Asset, AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { paraImplementation } from "../../parachains"
import { buildMessageId, resolveInputs } from "../../toEthereum_v2"
import { Context } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    buildContractCallHex,
    estimateFeesFromParachains,
    MaxWeight,
    mockDeliveryFee,
    validateTransfer,
    DeliveryFeeV2,
    TransferV2,
    ValidationResultV2,
} from "../../toEthereumSnowbridgeV2"
import { ConcreteToken } from "../../assets_v2"

export class PNAFromParachain implements TransferInterface {
    async getDeliveryFee(
        context: Context,
        sourceParaId: number,
        registry: AssetRegistry,
        tokenAddresses: string[],
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFeeV2> {
        const assetHub = await context.assetHub()
        const parachain = await context.parachain(sourceParaId)

        const sourceParachainImpl = await paraImplementation(parachain)
        const { sourceAssetMetadata } = resolveInputs(
            registry,
            tokenAddresses[0],
            sourceParachainImpl.parachainId,
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
            340282366920938463463374607431768211455n,
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
            mockDeliveryFee,
        )

        forwardedXcmToBH = buildExportXcm(
            assetHub.registry,
            registry.ethChainId,
            sourceAssetMetadata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            1n,
            1n,
        )

        const fees = await estimateFeesFromParachains(
            context,
            sourceParaId,
            registry,
            tokenAddresses,
            {
                localXcm,
                forwardXcmToAH,
                forwardedXcmToBH,
            },
            options,
        )
        return fees
    }

    async createTransfer(
        context: Context,
        sourceParaId: number,
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokens: ConcreteToken[],
        fee: DeliveryFeeV2,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<TransferV2> {
        const { ethChainId, assetHubParaId, environment } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const parachain = await context.parachain(sourceParaId)

        const sourceParachainImpl = await paraImplementation(parachain)
        const { tokenErcMetadata, sourceParachain, ahAssetMetadata, sourceAssetMetadata } =
            resolveInputs(registry, tokens[0].address, sourceParachainImpl.parachainId)

        let messageId: string | undefined = await buildMessageId(
            parachain,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            tokens[0].address,
            beneficiaryAccount,
            tokens[0].amount,
        )
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> = await this.createTx(
            context,
            parachain,
            environment,
            ethChainId,
            assetHubParaId,
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            beneficiaryAccount,
            sourceAssetMetadata,
            tokens[0].amount,
            messageId,
            fee,
            options,
        )

        return {
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokens,
                fee,
                contractCall: options?.contractCall,
            },
            computed: {
                sourceParaId: sourceParachainImpl.parachainId,
                sourceAccountHex,
                sourceParachain,
                aggregatedAssets: [
                    {
                        tokenErcMetadata,
                        ahAssetMetadata,
                        sourceAssetMetadata,
                        amount: tokens[0].amount,
                    },
                ],
                messageId,
            },
            tx,
        }
    }

    async validateTransfer(context: Context, transfer: TransferV2): Promise<ValidationResultV2> {
        return validateTransfer(context, transfer)
    }

    async createTx(
        context: Context,
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
        fee: DeliveryFeeV2,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<SubmittableExtrinsic<"promise", ISubmittableResult>> {
        let claimerLocation = options?.claimerLocation
        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(context, options.contractCall)
        }
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
                fee,
                claimerLocation,
                callHex,
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
                fee,
                claimerLocation,
                callHex,
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
                fee,
                claimerLocation,
                callHex,
            )
        } else {
            throw new Error(
                `Fee token as ${fee.feeLocation} is not supported. Only DOT or native asset is allowed.`,
            )
        }
        console.log("xcm on source chain:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
