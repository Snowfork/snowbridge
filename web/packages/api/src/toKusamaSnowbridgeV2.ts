import { TransferInterface } from "./transfers/toKusama/transferInterface"
import {
    Asset,
    AssetRegistry,
    ChainId,
    ERC20Metadata,
    EthereumChain,
    EthereumProviderTypes,
    MultiAddressStruct,
    Parachain,
    TransferKind,
    TransferRoute,
} from "@snowbridge/base-types"
import { FeeInfo, ValidationLog } from "./toPolkadot_v2"
import { OperationStatus } from "./status"
import { ensureValidationSuccess } from "./utils"
import { Context } from "./index"
import { messageId as getSharedMessageReceipt } from "./toPolkadotSnowbridgeV2"
import { ERC20ToKusamaAH } from "./transfers/toKusama/erc20ToKusamaAH"
import { PNAToKusamaAH } from "./transfers/toKusama/pnaToKusamaAH"

export type DeliveryFee = {
    kind: Extract<TransferKind, "ethereum->kusama">
    feeAsset: any
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    kusamaDeliveryFeeEther: bigint
    kusamaExecutionFeeEther: bigint
    relayerFee: bigint
    extrinsicFeeDot: bigint
    extrinsicFeeEther: bigint
    totalFeeInWei: bigint
}

export type Transfer<T extends EthereumProviderTypes> = {
    kind: Extract<TransferKind, "ethereum->kusama">
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        gatewayAddress: string
        beneficiaryAddressHex: string
        beneficiaryMultiAddress: MultiAddressStruct
        totalValue: bigint
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        kusamaAssetMetadata: Asset
        minimalBalance: bigint
        claimer: any
        topic: string
        totalInputAmount: bigint
    }
    tx: T["ContractTransaction"]
}

export type ValidatedTransfer<T extends EthereumProviderTypes> = Transfer<T> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        totalInputAmount?: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        kusamaDryRunError?: string
    }
}

export type MessageReceipt = {
    nonce: bigint
    payload: any
    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}

export class TransferToKusama<T extends EthereumProviderTypes> implements TransferInterface<T> {
    #pnaImpl?: TransferInterface<T>
    #erc20Impl?: TransferInterface<T>

    constructor(
        public readonly context: Context<T>,
        private readonly route: TransferRoute,
        private readonly registry: AssetRegistry,
        private readonly source: EthereumChain,
        private readonly destination: Parachain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    #resolveByTokenAddress(tokenAddress: string): TransferInterface<T> {
        const ahAssetMetadata =
            this.registry.parachains[`polkadot_${this.registry.assetHubParaId}`].assets[
                tokenAddress.toLowerCase()
            ]
        if (!ahAssetMetadata) {
            throw Error(`Token ${tokenAddress} not registered on asset hub.`)
        }

        if (ahAssetMetadata.location) {
            this.#pnaImpl ??= new PNAToKusamaAH(
                this.context,
                this.registry,
                this.route,
                this.source,
                this.destination,
            )
            return this.#pnaImpl
        }

        this.#erc20Impl ??= new ERC20ToKusamaAH(
            this.context,
            this.registry,
            this.route,
            this.source,
            this.destination,
        )
        return this.#erc20Impl
    }

    async fee(
        tokenAddress: string,
        options?: {
            paddFeeByPercentage?: bigint
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee> {
        return this.#resolveByTokenAddress(tokenAddress).fee(tokenAddress, options)
    }

    async tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer<T>> {
        return this.#resolveByTokenAddress(tokenAddress).tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        )
    }

    async build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                paddFeeByPercentage?: bigint
                overrideRelayerFee?: bigint
            }
        },
    ): Promise<ValidatedTransfer<T>> {
        const fee = await this.fee(tokenAddress, options?.fee)
        const transfer = await this.tx(
            sourceAccount,
            beneficiaryAccount,
            tokenAddress,
            amount,
            fee,
        )
        return ensureValidationSuccess(await this.validate(transfer))
    }

    async validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).validate(transfer)
    }

    async messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null> {
        return getSharedMessageReceipt(this.context.ethereumProvider, receipt)
    }
}
