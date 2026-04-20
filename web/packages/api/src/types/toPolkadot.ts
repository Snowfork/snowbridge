import type {
    Asset,
    AssetRegistry,
    ERC20Metadata,
    EthereumProviderTypes,
    FeeData,
    MultiAddressStruct,
    Parachain,
} from "@snowbridge/base-types"
import type { OperationStatus } from "../status"

export type Transfer<T extends EthereumProviderTypes> = {
    kind: "ethereum->polkadot"
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
        destAssetMetadata: Asset
        destParachain: Parachain
        destinationFeeInDOT: bigint
        minimalBalance: bigint
    }
    tx: T["ContractTransaction"]
}

export enum ValidationKind {
    Warning,
    Error,
}

export enum ValidationReason {
    MinimumAmountValidation,
    GatewaySpenderLimitReached,
    InsufficientTokenBalance,
    FeeEstimationError,
    InsufficientEther,
    BridgeStatusNotOperational,
    DryRunNotSupportedOnDestination,
    NoDestinationParachainConnection,
    DryRunFailed,
    MaxConsumersReached,
    AccountDoesNotExist,
}

export type ValidationLog = {
    kind: ValidationKind
    reason: ValidationReason
    message: string
}

export type FeeInfo = {
    estimatedGas: bigint
    feeData: FeeData
    executionFee: bigint
    totalTxCost: bigint
}

export type DeliveryFee = {
    kind: "ethereum->polkadot"
    destinationDeliveryFeeDOT: bigint
    destinationExecutionFeeDOT: bigint
    totalFeeInWei: bigint
}

export type ValidatedTransfer<T extends EthereumProviderTypes> = Transfer<T> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
    }
}

export type MessageReceipt = {
    channelId: string
    nonce: bigint
    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}
