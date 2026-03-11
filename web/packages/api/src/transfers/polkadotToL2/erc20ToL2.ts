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
    buildL2Call,
    estimateFeesFromAssetHub,
    MaxWeight,
    mockDeliveryFee,
    signAndSendTransfer,
    validateTransferFromAssetHub,
} from "../../toEthereumSnowbridgeV2"

export class ERC20FromAH<T extends EthereumProviderTypes> implements TransferInterface<T> {
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
        const context = this.context
        const registry = this.registry
        const assetHub = await context.assetHub()

        const sourceAssetMetadata = this.source.assets[tokenAddress.toLowerCase()]
        if (!sourceAssetMetadata) {
            throw Error(
                `Token ${tokenAddress} not registered on source parachain ${this.source.id}.`,
            )
        }

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
            this.to.id,
            amount,
        )
        return fees
    }

    async createTransfer(
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
        const context = this.context
        const registry = this.registry
        const { ethChainId } = registry

        let sourceAccountHex = sourceAccount
        if (!isHex(sourceAccountHex)) {
            sourceAccountHex = u8aToHex(decodeAddress(sourceAccount))
        }
        const parachain = await context.parachain(this.from.id)

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

        let callInfo = await buildL2Call(
            context,
            registry,
            tokenAddress,
            this.to.id,
            amount,
            beneficiaryAccount,
            messageId,
        )
        options = options || {}
        options.contractCall = options.contractCall || callInfo.l2Call

        const l1AdapterAddress = context.environment.l2Bridge?.l1AdapterAddress
        if (!l1AdapterAddress) {
            throw new Error("L2 bridge configuration is missing.")
        }

        let l1ReceiverAddress = l1AdapterAddress

        let callHex: string | undefined
        if (options?.contractCall) {
            callHex = await buildContractCallHex(context, options.contractCall)
        }
        let xcm: any
        if (!fee.feeLocation) {
            xcm = buildTransferXcmFromAssetHub(
                parachain.registry,
                ethChainId,
                sourceAccount,
                l1ReceiverAddress,
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
                l1ReceiverAddress,
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
                contractCall: options?.contractCall,
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
