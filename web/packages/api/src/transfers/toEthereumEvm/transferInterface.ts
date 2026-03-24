import { Context } from "../.."
import { ContractCall, EthereumProviderTypes } from "@snowbridge/base-types"
import type {
    MessageReceiptEvm,
    TransferEvm,
    ValidatedTransferEvm,
} from "../../types/toEthereumEvm"
import type { DeliveryFee } from "../../types/toEthereum"
export type {
    MessageReceiptEvm,
    TransferEvm,
    ValidatedTransferEvm,
} from "../../types/toEthereumEvm"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee>

    tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<TransferEvm<T>>

    validate(transfer: TransferEvm<T>): Promise<ValidatedTransferEvm<T>>

    build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
                slippagePadPercentage?: bigint
                defaultFee?: bigint
                feeTokenLocation?: any
                claimerLocation?: any
                contractCall?: ContractCall
            }
            tx?: {
                claimerLocation?: any
                contractCall?: ContractCall
            }
        },
    ): Promise<ValidatedTransferEvm<T>>

    messageId(receipt: T["TransactionReceipt"]): Promise<MessageReceiptEvm>
}
