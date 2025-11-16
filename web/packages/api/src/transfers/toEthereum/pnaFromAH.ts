import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { isRelaychainLocation } from "../../xcmBuilder"
import {
    buildExportXcm,
    buildTransferXcmFromAssetHub,
} from "../../xcmbuilders/toEthereum/pnaFromAH"
import { Asset, AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { paraImplementation } from "../../parachains"
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
import { ConcreteToken } from "src/assets_v2"

export class PNAFromAH implements TransferInterface {
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

        const { sourceAssetMetadata } = resolveInputs(registry, tokenAddresses[0], sourceParaId)

        let forwardedXcmToBH, localXcm: any

        localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            registry.ethChainId,
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
            ethChainId,
            sourceAccount,
            beneficiaryAccount,
            ahAssetMetadata,
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
        ethChainId: number,
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
            asset,
            amount,
            fee,
            callHex,
        )
        console.log("xcm on AH:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
