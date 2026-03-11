import { ApiPromise } from "@polkadot/api"
import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import { isHex, u8aToHex } from "@polkadot/util"
import { decodeAddress } from "@polkadot/util-crypto"
import { isRelaychainLocation } from "../../xcmBuilder"
import {
    buildExportXcm,
    buildTransferXcmFromAssetHub,
} from "../../xcmbuilders/toEthereum/pnaFromAH"
import { buildTransferXcmFromAssetHubWithDOTAsFee } from "../../xcmbuilders/toEthereum/pnaFromAHWithDotAsFee"
import {
    Asset,
    AssetRegistry,
    ChainId,
    ContractCall,
    EthereumChain,
    Parachain,
    TransferRoute,
} from "@snowbridge/base-types"
import {
    buildMessageId,
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidationResult,
} from "../../toEthereum_v2"
import { Context, EthereumProviderTypes } from "../.."
import { TransferInterface } from "./transferInterface"
import {
    buildContractCallHex,
    estimateFeesFromAssetHub,
    MaxWeight,
    mockDeliveryFee,
    signAndSendTransfer,
    validateTransferFromAssetHub,
} from "../../toEthereumSnowbridgeV2"

export class PNAFromAH<T extends EthereumProviderTypes> implements TransferInterface<T> {
    constructor(
        public readonly context: Context<T>,
        public readonly registry: AssetRegistry,
        public readonly route: TransferRoute,
        public readonly source: Parachain,
        public readonly destination: EthereumChain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    async getDeliveryFee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee> {
        const assetHub = await this.context.assetHub()
        const parachain = await this.context.parachain(this.from.id)

        const sourceParachainImpl = await this.context.paraImplementation(parachain)
        const sourceAssetMetadata = this.source.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on source parachain ${this.source.id}.`,
            )
        }

        let forwardedXcmToBH, localXcm: any

        localXcm = buildTransferXcmFromAssetHub(
            assetHub.registry,
            this.registry.ethChainId,
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
        const parachain = await this.context.parachain(this.from.id)

        const sourceParachainImpl = await this.context.paraImplementation(parachain)
        const tokenErcMetadata =
            registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!tokenErcMetadata) {
            throw Error(
                `No token ${tokenAddress} registered on ethereum chain ${registry.ethChainId}.`,
            )
        }
        const ahAssetMetadata =
            registry.parachains[`polkadot_${registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on asset hub.`)
        }
        const sourceParachain = this.source
        const sourceAssetMetadata = sourceParachain.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on source parachain ${sourceParachain.id}.`,
            )
        }

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
