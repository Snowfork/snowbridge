import type { SubmittableExtrinsic } from "@polkadot/api/types"
import type { ISubmittableResult } from "@polkadot/types/types"
import type { Asset, AssetRegistry, Parachain } from "@snowbridge/base-types"
import type { EventRecord } from "@polkadot/types/interfaces"
import type { FeeAsset, FeeItem, KusamaFeeKey } from "./fee"

export type Transfer = {
    kind: "kusama->polkadot" | "polkadot->kusama"
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
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
        messageId?: string
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    kind: "kusama->polkadot" | "polkadot->kusama"
    xcmBridgeFee: bigint
    bridgeHubDeliveryFee: bigint
    destinationFee: bigint
    totalFeeInNative: bigint
    breakdown: { [P in KusamaFeeKey]?: FeeAsset[] }
    summary: FeeItem[]
    totals: FeeAsset[]
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
        assetHubDryRunError: any
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
