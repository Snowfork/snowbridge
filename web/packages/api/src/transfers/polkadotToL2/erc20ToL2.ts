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
import { Asset, AssetRegistry, ContractCall } from "@snowbridge/base-types"
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
    buildContractCallHex,
    buildL2Call,
    estimateFeesFromAssetHub,
    MaxWeight,
    mockDeliveryFee,
    validateTransferFromAssetHub,
} from "../../toEthereumSnowbridgeV2"

export class ERC20FromAH implements TransferInterface {
    async getDeliveryFee(
        context: Context,
        registry: AssetRegistry,
        l2ChainId: number,
        tokenAddress: string,
        amount: bigint,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            contractCall?: ContractCall
            l2PadFeeByPercentage?: bigint
            fillDeadlineBuffer?: bigint
        },
    ): Promise<DeliveryFee> {
        const assetHub = await context.assetHub()

        const { sourceAssetMetadata } = resolveInputs(
            registry,
            tokenAddress,
            registry.assetHubParaId,
        )

        let localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            mockDeliveryFee,
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
            context,
            registry,
            tokenAddress,
            {
                localXcm,
                forwardedXcmToBH,
            },
            options,
            l2ChainId,
            amount,
        )
        return fees
    }

    async createTransfer(
        context: Context,
        registry: AssetRegistry,
        l2ChainId: number,
        tokenAddress: string,
        amount: bigint,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<Transfer> {
        const { ethChainId } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const parachain = await context.parachain(registry.assetHubParaId)

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

        let callInfo = await buildL2Call(
            context,
            registry,
            tokenAddress,
            l2ChainId,
            amount,
            beneficiaryAccount,
            messageId,
        )
        options = options || {}
        options.contractCall = options.contractCall || callInfo.l2Call

        const l1AdapterAddress = await context.l1Adapter().getAddress()

        let l1ReceiverAddress = l1AdapterAddress

        let tx: SubmittableExtrinsic<"promise", ISubmittableResult>

        tx = await this.createTx(
            context,
            parachain,
            ethChainId,
            sourceAccount,
            l1ReceiverAddress,
            ahAssetMetadata,
            amount,
            messageId,
            fee,
            options,
        )

        return {
            input: {
                registry,
                sourceAccount,
                beneficiaryAccount,
                tokenAddress,
                amount,
                fee,
                contractCall: options?.contractCall,
            },
            computed: {
                sourceParaId: sourceParachainImpl.parachainId,
                sourceAccountHex,
                tokenErcMetadata,
                sourceParachain,
                ahAssetMetadata,
                sourceAssetMetadata,
                messageId,
                contractCall: options?.contractCall,
            },
            tx,
        }
    }

    async validateTransfer(context: Context, transfer: Transfer): Promise<ValidationResult> {
        return validateTransferFromAssetHub(context, transfer)
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
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<SubmittableExtrinsic<"promise", ISubmittableResult>> {
        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(context, options.contractCall)
        }
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
                fee,
                callHex,
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
                fee,
                callHex,
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on AH:", xcm.toHuman())
        return parachain.tx.polkadotXcm.execute(xcm, MaxWeight)
    }
}
