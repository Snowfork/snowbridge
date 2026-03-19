import { EthereumProviderTypes } from "@snowbridge/base-types"
import { Context } from "../.."
import {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "../../toPolkadotSnowbridgeV2"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        amount: bigint,
        options?: {
            padFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
            l2PadFeeByPercentage?: bigint
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
    ): Promise<DeliveryFee>

    tx(
        tokenAddress: string,
        amount: bigint,
        sourceAccount: string,
        beneficiaryAccount: string,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[] // Optional custom XCM instructions to append
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
    ): Promise<Transfer<T>>

    validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>>

    build(
        tokenAddress: string,
        amount: bigint,
        sourceAccount: string,
        beneficiaryAccount: string,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
                feeAsset?: any
                customXcm?: any[]
                overrideRelayerFee?: bigint
                l2PadFeeByPercentage?: bigint
                fillDeadlineBuffer?: bigint
            }
            tx?: {
                customXcm?: any[]
                fillDeadlineBuffer?: bigint
            }
        },
    ): Promise<ValidatedTransfer<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}
