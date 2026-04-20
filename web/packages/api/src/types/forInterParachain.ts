import type { SubmittableExtrinsic } from "@polkadot/api/types"
import type { ISubmittableResult } from "@polkadot/types/types"
import type { Asset, AssetRegistry, Parachain } from "@snowbridge/base-types"
import type { EventRecord } from "@polkadot/types/interfaces"

export type Transfer = {
    kind: "polkadot->polkadot"
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        beneficiaryAddressHex: string
        sourceAccountHex: string
        sourceAssetMetadata: Asset
        destAssetMetadata: Asset
        sourceParachain: Parachain
        destParachain: Parachain
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    kind: "polkadot->polkadot"
    deliveryFee: bigint
    executionFee: bigint
    totalFeeInDot: bigint
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    InsufficientTokenBalance,
    DryRunFailed,
    MinimumAmountValidation,
    InsufficientFee,
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
