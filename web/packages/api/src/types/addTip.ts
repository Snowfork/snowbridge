import type { SubmittableExtrinsic } from "@polkadot/api/types"
import type { ISubmittableResult } from "@polkadot/types/types"

export type MessageDirection = "Inbound" | "Outbound"

export type TipAsset = "DOT" | "ETH"

export type TipAdditionParams = {
    direction: MessageDirection
    nonce: bigint
    tipAsset: TipAsset
    tipAmount: bigint
}

export type TipAddition = {
    input: TipAdditionParams
    computed: {
        tipAssetLocation: any
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export enum TipAdditionValidationKind {
    Warning,
    Error,
}

export type TipAdditionValidationLog = {
    kind: TipAdditionValidationKind
    message: string
}

export type ValidatedTipAddition = TipAddition & {
    logs: TipAdditionValidationLog[]
    success: boolean
    data: {
        extrinsicFee: bigint
        errorMessage?: string
        executionResult?: any
    }
}

export type TipAdditionResponse = {
    blockHash: string
    txHash: string
}
