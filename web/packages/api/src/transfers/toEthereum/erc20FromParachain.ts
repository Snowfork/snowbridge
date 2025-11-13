import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { DOT_LOCATION, isRelaychainLocation, isParachainNative } from "../../xcmBuilder"
import { buildExportXcm } from "../../xcmbuilders/toEthereum/erc20FromAH"
import {
    buildResultXcmAssetHubERC20TransferFromParachain,
    buildTransferXcmFromParachain,
} from "../../xcmbuilders/toEthereum/erc20FromParachain"
import { buildTransferXcmFromParachainWithDOTAsFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithDotAsFee"
import { buildTransferXcmFromParachainWithNativeAssetFee } from "../../xcmbuilders/toEthereum/erc20FromParachainWithNativeAsFee"
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
import { AggregatedAsset, ConcreteAsset, ConcreteToken } from "src/assets_v2"

export class ERC20FromParachain implements TransferInterface {
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

        let concreteAssets: ConcreteAsset[] = []
        for (const tokenAddress of tokenAddresses) {
            const { sourceAssetMetadata } = resolveInputs(registry, tokenAddress, sourceParaId)
            concreteAssets.push({
                id: sourceAssetMetadata,
                amount: 1n,
            })
        }

        let forwardXcmToAH: any, forwardedXcmToBH: any, localXcm: any

        forwardXcmToAH = buildResultXcmAssetHubERC20TransferFromParachain(
            assetHub.registry,
            registry.ethChainId,
            sourceParaId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            concreteAssets,
            mockDeliveryFee,
        )

        localXcm = buildTransferXcmFromParachain(
            assetHub.registry,
            registry.environment,
            registry.ethChainId,
            registry.assetHubParaId,
            sourceParaId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            concreteAssets,
            mockDeliveryFee,
        )

        forwardedXcmToBH = buildExportXcm(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            concreteAssets,
            mockDeliveryFee,
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
        let sourceParachain = registry.parachains[sourceParaId.toString()]
        let concreteAssets: ConcreteAsset[] = [],
            aggregatedAssets: AggregatedAsset[] = []

        for (const token of tokens) {
            const { tokenErcMetadata, ahAssetMetadata, sourceAssetMetadata } = resolveInputs(
                registry,
                token.address,
                sourceParaId,
            )
            concreteAssets.push({
                id: sourceAssetMetadata,
                amount: token.amount,
            })
            aggregatedAssets.push({
                tokenErcMetadata,
                ahAssetMetadata,
                sourceAssetMetadata,
                amount: token.amount,
            })
        }

        let messageId = await buildMessageId(
            parachain,
            sourceParaId,
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
            sourceParaId,
            sourceAccountHex,
            beneficiaryAccount,
            messageId,
            concreteAssets,
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
                sourceParaId,
                sourceAccountHex,
                sourceParachain,
                aggregatedAssets,
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
        messageId: string,
        concreteAssets: ConcreteAsset[],
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
                concreteAssets,
                fee,
                claimerLocation,
                callHex,
            )
        } // One swap from DOT to Ether on Asset Hub.
        else if (isRelaychainLocation(fee.feeLocation)) {
            xcm = buildTransferXcmFromParachain(
                parachain.registry,
                envName,
                ethChainId,
                assetHubParaId,
                sourceParachainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                concreteAssets,
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
                concreteAssets,
                fee,
                claimerLocation,
                callHex,
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on source chain:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
