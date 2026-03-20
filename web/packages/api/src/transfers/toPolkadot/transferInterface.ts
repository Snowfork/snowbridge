import { EthereumProviderTypes } from "@snowbridge/base-types"
import { Context } from "../.."
import type { MessageReceipt as ToPolkadotV1MessageReceipt } from "../../types/toPolkadot"
import type {
    DeliveryFee,
    MessageReceipt as ToPolkadotV2MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "../../types/toPolkadotSnowbridgeV2"

export type MessageReceipt = ToPolkadotV1MessageReceipt | ToPolkadotV2MessageReceipt

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee>

    tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[], // Optional custom XCM instructions to append
    ): Promise<Transfer<T>>

    validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>>

    build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
                feeAsset?: any
                customXcm?: any[]
                overrideRelayerFee?: bigint
            }
            customXcm?: any[]
        },
    ): Promise<ValidatedTransfer<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}
