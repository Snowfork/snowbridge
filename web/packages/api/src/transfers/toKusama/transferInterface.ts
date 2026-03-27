import { EthereumProviderTypes } from "@snowbridge/base-types"
import { Context } from "../.."
import type { MessageReceipt as ToPolkadotV2MessageReceipt } from "../../toPolkadotSnowbridgeV2"
import { DeliveryFee } from "../../toKusamaSnowbridgeV2"
import type { Transfer, ValidatedTransfer } from "../../toKusamaSnowbridgeV2"

export type MessageReceipt = ToPolkadotV2MessageReceipt

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            paddFeeByPercentage?: bigint
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee>

    tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer<T>>

    validate(transfer: Transfer<T>): Promise<ValidatedTransfer<T>>

    build(
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
    ): Promise<ValidatedTransfer<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}
