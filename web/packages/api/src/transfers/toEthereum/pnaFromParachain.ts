import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
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
import { Asset, AssetRegistry, ChainId, ContractCall } from "@snowbridge/base-types"
import { paraImplementation } from "../../parachains"
import {
    buildMessageId,
    DeliveryFee,
    MessageReceipt,
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
    signAndSendTransfer,
    validateTransferFromParachain,
} from "../../toEthereumSnowbridgeV2"

export class PNAFromParachain implements TransferInterface {
    constructor(
        public readonly context: EthersContext,
        public readonly registry: AssetRegistry,
        public readonly from: ChainId,
        public readonly to: ChainId,
    ) {}

    async getDeliveryFee(
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
        const assetHub = await this.context.assetHub()
        const parachain = await this.context.parachain(this.from.id)

        const sourceParachainImpl = await paraImplementation(parachain)
        const { sourceAssetMetadata } = resolveInputs(
            this.registry,
            tokenAddress,
            sourceParachainImpl.parachainId,
        )

        let forwardXcmToAH: any, forwardedXcmToBH: any, returnToSenderXcm: any, localXcm: any

        forwardXcmToAH = buildResultXcmAssetHubPNATransferFromParachain(
            assetHub.registry,
            this.registry.ethChainId,
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
            this.registry.environment,
            this.registry.ethChainId,
            this.registry.assetHubParaId,
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
            this.registry.ethChainId,
            sourceAssetMetadata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            1n,
            1n,
        )

        const fees = await estimateFeesFromParachains(
            this.context,
            this.from.id,
            this.registry,
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
        const registry = this.registry
        const { ethChainId, assetHubParaId, environment } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const parachain = await this.context.parachain(this.from.id)

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
        let claimerLocation = options?.claimerLocation
        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(this.context, options.contractCall)
        }
        let xcm: any
        if (!fee.feeLocation) {
            xcm = buildTransferXcmFromParachain(
                parachain.registry,
                environment,
                ethChainId,
                assetHubParaId,
                sourceParachainImpl.parachainId,
                sourceAccountHex,
                beneficiaryAccount,
                messageId,
                sourceAssetMetadata,
                amount,
                fee,
                claimerLocation,
                callHex,
            )
        } else if (isRelaychainLocation(fee.feeLocation)) {
            xcm = buildTransferXcmFromParachainWithDOTAsFee(
                parachain.registry,
                environment,
                ethChainId,
                assetHubParaId,
                sourceParachainImpl.parachainId,
                sourceAccountHex,
                beneficiaryAccount,
                messageId,
                sourceAssetMetadata,
                amount,
                fee,
                claimerLocation,
                callHex,
            )
        } else if (isParachainNative(fee.feeLocation, sourceParachainImpl.parachainId)) {
            xcm = buildTransferXcmFromParachainWithNativeAssetFee(
                parachain.registry,
                environment,
                ethChainId,
                assetHubParaId,
                sourceParachainImpl.parachainId,
                sourceAccountHex,
                beneficiaryAccount,
                messageId,
                sourceAssetMetadata,
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
        let tx: SubmittableExtrinsic<"promise", ISubmittableResult> =
            parachain.tx.polkadotXcm.execute(xcm, MaxWeight)

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

    async validateTransfer(transfer: Transfer): Promise<ValidationResult> {
        return validateTransferFromParachain(this.context, transfer)
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        const sourceParachain = await this.context.parachain(transfer.computed.sourceParaId)
        return signAndSendTransfer(sourceParachain, transfer, account, options)
    }
}
