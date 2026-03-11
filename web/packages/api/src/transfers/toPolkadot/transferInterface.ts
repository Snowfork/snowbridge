import { Context, EthereumProviderTypes } from "../.."
import type { MessageReceipt as ToPolkadotV1MessageReceipt } from "../../toPolkadot_v2"
import { DeliveryFee } from "../../toPolkadotSnowbridgeV2"
import type { MessageReceipt as ToPolkadotV2MessageReceipt } from "../../toPolkadotSnowbridgeV2"
import { Transfer } from "../../toPolkadotSnowbridgeV2"
import { ValidationResult } from "../../toPolkadotSnowbridgeV2"

export type MessageReceipt = ToPolkadotV1MessageReceipt | ToPolkadotV2MessageReceipt

export interface TransferInterface<T extends EthereumProviderTypes = EthereumProviderTypes> {
    readonly context: Context<T>

    getDeliveryFee(
        tokenAddress: string,
        options?: {
            paddFeeByPercentage?: bigint
            feeAsset?: any
            customXcm?: any[] // Optional custom XCM instructions to append
            overrideRelayerFee?: bigint
        },
    ): Promise<DeliveryFee>

    createTransfer(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        customXcm?: any[], // Optional custom XCM instructions to append
    ): Promise<Transfer>

    validateTransfer(transfer: Transfer): Promise<ValidationResult>

    getMessageReceipt(receipt: T["TransactionReceipt"]): Promise<MessageReceipt | null>
}
