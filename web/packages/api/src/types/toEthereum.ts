import type { SubmittableExtrinsic } from "@polkadot/api/types"
import type { ISubmittableResult } from "@polkadot/types/types"
import type {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    FeeData,
    Parachain,
} from "@snowbridge/base-types"
import type { EventRecord } from "@polkadot/types/interfaces"
import type { OperationStatus } from "../status"

export type Transfer = {
    kind: "polkadot->ethereum" | "polkadot->ethereum_l2"
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
        contractCall?: ContractCall
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId?: string
        contractCall?: ContractCall
    }
    tx: SubmittableExtrinsic<"promise", ISubmittableResult>
}

export type DeliveryFee = {
    kind: "polkadot->ethereum" | "polkadot->ethereum_l2" | "ethereum->ethereum"
    snowbridgeDeliveryFeeDOT: bigint
    bridgeHubDeliveryFeeDOT: bigint
    assetHubExecutionFeeDOT: bigint
    returnToSenderExecutionFeeDOT: bigint
    returnToSenderDeliveryFeeDOT: bigint
    totalFeeInDot: bigint
    localExecutionFeeDOT?: bigint
    localDeliveryFeeDOT?: bigint
    ethereumExecutionFee?: bigint
    feeLocation?: any
    totalFeeInNative?: bigint
    assetHubExecutionFeeNative?: bigint
    returnToSenderExecutionFeeNative?: bigint
    localExecutionFeeInNative?: bigint
    localDeliveryFeeInNative?: bigint
    ethereumExecutionFeeInNative?: bigint
    l2BridgeFeeInL1Token?: bigint
    volumeTip?: bigint
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    InsufficientTokenBalance,
    FeeEstimationError,
    DryRunApiNotAvailable,
    DryRunFailed,
    InsufficientDotFee,
    BridgeStatusNotOperational,
    InsufficientNativeFee,
    InsufficientEtherBalance,
    ContractCallInvalidTarget,
    ContractCallAgentNotRegistered,
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
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance?: bigint
        sourceExecutionFee?: bigint
        tokenBalance: bigint
        feeInfo?: FeeInfo
        sourceDryRunError: any
        assetHubDryRunError: any
        bridgeHubDryRunError?: any
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
