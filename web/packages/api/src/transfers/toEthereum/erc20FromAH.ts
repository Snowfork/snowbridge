import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
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
    MessageReceipt,
    resolveInputs,
    Transfer,
    ValidationResult,
} from "../../toEthereum_v2"
import { EthersContext } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    buildContractCallHex,
    estimateFeesFromAssetHub,
    MaxWeight,
    mockDeliveryFee,
    signAndSendTransfer,
    validateTransferFromAssetHub,
} from "../../toEthereumSnowbridgeV2"

export class ERC20FromAH implements TransferInterface {
    constructor(
        private readonly context: EthersContext,
        private readonly registry: AssetRegistry,
    ) {}

    async getDeliveryFee(
        source: { sourceParaId: number },
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee> {
        const { assetHub } =
            "sourceParaId" in source
                ? {
                      assetHub: await this.context.assetHub(),
                  }
                : source

        const { sourceAssetMetadata } = resolveInputs(
            this.registry,
            tokenAddress,
            source.sourceParaId,
        )

        let localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            this.registry.ethChainId,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            sourceAssetMetadata,
            1n,
            mockDeliveryFee,
        )

        let forwardedXcmToBH = buildExportXcm(
            assetHub.registry,
            this.registry.ethChainId,
            sourceAssetMetadata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            1n,
            1n,
        )

        const fees = await estimateFeesFromAssetHub(
            this.context,
            this.registry,
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
        source: { sourceParaId: number },
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
        const { ethChainId } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const { parachain } =
            "sourceParaId" in source
                ? { parachain: await this.context.parachain(source.sourceParaId) }
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
        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(this.context, options.contractCall)
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
                ahAssetMetadata,
                amount,
                fee,
                callHex,
            )
        } else if (isRelaychainLocation(fee.feeLocation)) {
            xcm = buildTransferXcmFromAssetHubWithDOTAsFee(
                parachain.registry,
                ethChainId,
                sourceAccount,
                beneficiaryAccount,
                messageId,
                ahAssetMetadata,
                amount,
                fee,
                callHex,
            )
        } else {
            throw new Error(`Fee token as ${fee.feeLocation} is not supported yet.`)
        }
        console.log("xcm on AH:", xcm.toHuman())
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
        return validateTransferFromAssetHub(this.context, transfer)
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
