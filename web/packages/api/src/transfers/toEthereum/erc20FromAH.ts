import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import {
    buildExportXcm,
    buildTransferXcmFromAssetHub,
} from "../../xcmbuilders/toEthereum/erc20FromAH"
import { AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { buildMessageId, resolveInputs } from "../../toEthereum_v2"
import { Context } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    buildContractCallHex,
    estimateFeesFromAssetHub,
    MaxWeight,
    mockDeliveryFee,
    validateTransfer,
    DeliveryFeeV2,
    TransferV2,
    ValidationResultV2,
} from "../../toEthereumSnowbridgeV2"
import { AggregatedAsset, ConcreteAsset, ConcreteToken } from "../../assets_v2"

export class ERC20FromAH implements TransferInterface {
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

        let localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            concreteAssets,
            mockDeliveryFee,
        )

        let forwardedXcmToBH = buildExportXcm(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            concreteAssets,
            mockDeliveryFee,
        )

        const fees = await estimateFeesFromAssetHub(
            context,
            registry,
            tokenAddresses,
            {
                localXcm,
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
        const { ethChainId } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const assetHub = await context.assetHub()

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
            assetHub,
            sourceParaId,
            sourceAccountHex,
            tokens[0].address,
            beneficiaryAccount,
            tokens[0].amount,
        )
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> = await this.createTx(
            context,
            assetHub,
            ethChainId,
            sourceAccount,
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
                sourceParaId: sourceParaId,
                sourceAccountHex,
                aggregatedAssets,
                sourceParachain,
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
        ethChainId: number,
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
        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(context, options.contractCall)
        }
        const xcm = buildTransferXcmFromAssetHub(
            parachain.registry,
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            messageId,
            concreteAssets,
            fee,
            callHex,
        )
        console.log("xcm on AH:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
