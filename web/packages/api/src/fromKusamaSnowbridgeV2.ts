import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"
import {
    Asset,
    AssetRegistry,
    ChainId,
    ERC20Metadata,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
    TransferKind,
    TransferRoute,
} from "@snowbridge/base-types"
import { EventRecord } from "@polkadot/types/interfaces"
import { ensureValidationSuccess } from "./utils"
import { TransferInterface } from "./transfers/fromKusama/transferInterface"
import { Context } from "./index"
import { ERC20FromKusamaAH } from "./transfers/fromKusama/erc20FromKusamaAH"

export type Transfer = {
    kind: Extract<TransferKind, "kusama->ethereum">
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    kind: Extract<TransferKind, "kusama->ethereum">
    kusamaBridgeFee: bigint
    kusamaBridgeHubDeliveryFee: bigint
    polkadotAHExecutionFee: bigint
    snowbridgeDeliveryFee: bigint
    ethereumExecutionFee: bigint
    totalFeeInDOT: bigint
    totalFeeInKSM: bigint
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    InsufficientTokenBalance,
    InsufficientFee,
    DryRunFailed,
    MaxConsumersReached,
    AccountDoesNotExist,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type ValidatedTransfer = Transfer & {
    logs: ValidationLog[]
    success: boolean
    data: {
        nativeBalance: bigint
        sourceExecutionFee: bigint
        tokenBalance: bigint
        dryRunError: any
        polkadotAHDryRunError?: string
    }
}

export type MessageReceipt = {
    blockNumber: number
    blockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
}

export class TransferFromKusama<T extends EthereumProviderTypes> implements TransferInterface<T> {
    #erc20Impl?: TransferInterface<T>

    constructor(
        public readonly context: Context<T>,
        private readonly route: TransferRoute,
        private readonly registry: AssetRegistry,
        private readonly source: Parachain,
        private readonly destination: EthereumChain,
    ) {}

    get from(): ChainId {
        return this.route.from
    }

    get to(): ChainId {
        return this.route.to
    }

    #resolveByTokenAddress(_tokenAddress: string): TransferInterface<T> {
        this.#erc20Impl ??= new ERC20FromKusamaAH(
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
            padPercentage?: bigint
            slippagePadPercentage?: bigint
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
    ): Promise<Transfer> {
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
                padPercentage?: bigint
                slippagePadPercentage?: bigint
            }
        },
    ): Promise<ValidatedTransfer> {
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

    async validate(transfer: Transfer): Promise<ValidatedTransfer> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).validate(transfer)
    }

    async signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt> {
        return this.#resolveByTokenAddress(transfer.input.tokenAddress).signAndSend(
            transfer,
            account,
            options,
        )
    }
}
