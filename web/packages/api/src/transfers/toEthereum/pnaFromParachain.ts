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
import { Asset, AssetRegistry, ContractCall } from "@snowbridge/base-types"
import { paraImplementation } from "../../parachains"
import {
    buildMessageId,
    DeliveryFee,
    resolveInputs,
    Transfer,
    ValidationResult,
} from "../../toEthereum_v2"
import { EthersContext } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    buildContractCallHex,
    estimateFeesFromParachains,
    MaxWeight,
    mockDeliveryFee,
    validateTransferFromParachain,
} from "../../toEthereumSnowbridgeV2"

export class PNAFromParachain implements TransferInterface {
    async getDeliveryFee(
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
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

        returnToSenderXcm = buildParachainPNAReceivedXcmOnDestination(
            parachain.registry,
            sourceAssetMetadata.location,
            340282366920938463463374607431768211455n,
            340282366920938463463374607431768211455n,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
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
            options,
        )
        return fees
    }

    async createTransfer(
        source: { sourceParaId: number; context: EthersContext },
        registry: AssetRegistry,
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
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

        const accountNonce = await sourceParachainImpl.accountNonce(sourceAccountHex)
        let messageId: string | undefined = buildMessageId(
            sourceParachainImpl.parachainId,
            sourceAccountHex,
            accountNonce,
            tokenAddress,
            beneficiaryAccount,
            amount,
        )
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> = await this.createTx(
            source.context,
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
            },
            tx,
        }
    }

    async validateTransfer(context: EthersContext, transfer: Transfer): Promise<ValidationResult> {
        return validateTransferFromParachain(context, transfer)
    }

    async createTx(
        context: EthersContext,
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
