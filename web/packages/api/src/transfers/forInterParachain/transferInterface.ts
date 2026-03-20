import { Context } from "../.."
import { AddressOrPair, SignerOptions } from "@polkadot/api/types"
import { EthereumProviderTypes } from "@snowbridge/base-types"
import type {
    DeliveryFee,
    MessageReceipt,
    Transfer,
    ValidatedTransfer,
} from "../../types/forInterParachain"

export interface TransferInterface<T extends EthereumProviderTypes> {
    readonly context: Context<T>

    fee(
        tokenAddress: string,
        options?: {
            padFeeByPercentage?: bigint
        },
    ): Promise<DeliveryFee>

    tx(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
    ): Promise<Transfer>

    validate(transfer: Transfer): Promise<ValidatedTransfer>

    build(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        options?: {
            fee?: {
                padFeeByPercentage?: bigint
            }
        },
    ): Promise<ValidatedTransfer>

    signAndSend(
        transfer: Transfer,
        account: AddressOrPair,
        options: Partial<SignerOptions>,
    ): Promise<MessageReceipt>
}
