import { EthereumProviderTypes } from "@snowbridge/base-types"
import { Context } from "../.."
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "../../types/toPolkadotSnowbridgeV2"
import type { VolumeFeeParams } from "../../feeSchedule"

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
            volumeFee?: VolumeFeeParams
        },
    ): Promise<DeliveryFee>

    tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            customXcm?: any[] // Optional custom XCM instructions to append
            fillDeadlineBuffer?: bigint // Optional buffer added to the relay fill deadline for L2 calls.
        },
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
                l2PadFeeByPercentage?: bigint
                fillDeadlineBuffer?: bigint
                volumeFee?: VolumeFeeParams
            }
            tx?: {
                customXcm?: any[]
                fillDeadlineBuffer?: bigint
            }
        },
    ): Promise<ValidatedTransfer<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}
